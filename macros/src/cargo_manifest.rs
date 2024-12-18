use proc_macro::TokenStream;
use std::{collections::HashMap, env, path::PathBuf, sync::LazyLock};
use toml_edit::{DocumentMut, Item};
use tracing::{info, trace};
use thiserror::Error;

pub fn pretty_format_syn_path(path: syn::Path) -> String {
  let mut path_str = String::new();
  let has_leading_colon = path.leading_colon.is_some();
  if has_leading_colon {
    path_str.push_str("::");
  }
  for segment in &path.segments {
    path_str.push_str(&segment.ident.to_string());
    path_str.push_str("::");
  }
  path_str.truncate(path_str.len() - 2);
  path_str
}

pub type PathPiece = syn::punctuated::Punctuated<syn::PathSegment, syn::Token![::]>;

pub trait CrateReExportingPolicy {
  fn get_re_exported_crate_path(&self, crate_name: &str) -> Option<PathPiece>;
}

pub struct KnownReExportingCrate<'a> {
  re_exporting_crate_name: &'a str,
  crate_re_exporting_policy: &'a dyn CrateReExportingPolicy,
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum TryResolveCratePathError {
  #[error("Ambiguous crate dependency {0}.")]
  AmbiguousDependency(String),
}

/// The key is the crate name.
/// The value is the absolute module path of the crate.
struct ResolvedDependenciesMap(HashMap<String, Result<String, TryResolveCratePathError>>);

/// The cargo manifest (`Cargo.toml`) of the crate that is being built.
/// This can be the user's crate that either directly or indirectly depends on your crate.
/// If there are uses of the proc macro in your own crate, it may also point to the manifest of your own crate.
///
/// The [`CargoManifest`] is used to resolve a crate name to an absolute module path.
pub struct CargoManifest {
  cargo_manifest_path: PathBuf,
  cargo_manifest: DocumentMut,

  resolved_dependencies: ResolvedDependenciesMap,
  resolved_dependencies_dev: ResolvedDependenciesMap,
  resolved_dependencies_build: ResolvedDependenciesMap,
}

fn resolve_dependencies(dependencies_section: &Item, resolved_dependencies: &mut ResolvedDependenciesMap) {
  let dependencies_table = dependencies_section
    .as_table()
    .expect("The dependencies section in the Cargo.toml is not a table");

  for (dependency_key, dependency_item) in dependencies_table {
    // Get the actual dependency name whether it is remapped or not
    let dependency_crate_name = *dependency_item.get("package").map(|package_field| {
      package_field
        .as_str()
        .expect("The package name in the Cargo.toml is not a string")
    }).get_or_insert(dependency_key);

    // Try to insert the crate into the [`ResolvedDependencyMap`].
    let insert_result = resolved_dependencies.0.try_insert(dependency_crate_name.to_string(), Ok(dependency_key.to_string()));
    if let Err(_) = insert_result {
      // If the crate name is occupied.
      // We have an ambiguous dependency whose path can't be resolved.
      resolved_dependencies.0.insert(dependency_crate_name.to_string(), Err(TryResolveCratePathError::AmbiguousDependency(dependency_crate_name.to_string())));
    }
  }
}

impl CargoManifest {
  /// Returns a global shared instance of the [`CargoManifest`] struct.
  pub fn shared() -> &'static LazyLock<Self> {
    static LAZY_MANIFEST: LazyLock<CargoManifest> = LazyLock::new(|| {
      let cargo_manifest_dir = env::var_os("CARGO_MANIFEST_DIR");
      info!("CARGO_MANIFEST_DIR: {:?}", cargo_manifest_dir);

      let cargo_manifest_path = cargo_manifest_dir
        .map(PathBuf::from)
        .map(|mut path| {
          path.push("Cargo.toml");
          if !path.exists() {
            panic!(
              "No Cargo manifest found for crate. Expected: {}",
              path.display()
            );
          }
          path
        })
        .expect("CARGO_MANIFEST_DIR is not defined.");

      let cargo_manifest_string = std::fs::read_to_string(cargo_manifest_path.clone())
        .unwrap_or_else(|err| {
          panic!(
            "Unable to read cargo manifest: {} - {err}",
            cargo_manifest_path.display()
          )
        });

      let cargo_manifest = cargo_manifest_string
        .parse::<DocumentMut>()
        .unwrap_or_else(|err| {
          panic!(
            "Failed to parse cargo manifest: {} - {err}",
            cargo_manifest_path.display()
          )
        });

      let dependencies_section = cargo_manifest.get("dependencies");
      let dependencies_section_dev = cargo_manifest.get("dev-dependencies");
      let dependencies_section_build = cargo_manifest.get("build-dependencies");

      let mut resolved_dependencies = ResolvedDependenciesMap(HashMap::new());
      let mut resolved_dependencies_dev = ResolvedDependenciesMap(HashMap::new());
      let mut resolved_dependencies_build = ResolvedDependenciesMap(HashMap::new());

      if let Some(dependencies_section) = dependencies_section {
        resolve_dependencies(dependencies_section, &mut resolved_dependencies);
      }
      if let Some(dependencies_section_dev) = dependencies_section_dev {
        resolve_dependencies(dependencies_section_dev, &mut resolved_dependencies_dev);
      }
      if let Some(dependencies_section_build) = dependencies_section_dev {
        resolve_dependencies(dependencies_section_build, &mut resolved_dependencies_build);
      }

      CargoManifest {
        cargo_manifest_path,
        cargo_manifest,
        resolved_dependencies,
        resolved_dependencies_dev,
        resolved_dependencies_build,
      }
    });
    &LAZY_MANIFEST
  }

  pub fn try_resolve_crate_path_for_dependency_map(&self, crate_name: &str, known_re_exporting_crates: &[&KnownReExportingCrate<'_>], resolved_dependencies: &ResolvedDependenciesMap) -> Result<syn::Path, TryResolveCratePathError> {
  }

  /// Attempt to retrieve the absolute module path of a crate named [possible_crate_names](str) as an absolute [`syn::Path`].
  /// Remapped crate names are also supported.
  ///
  ///  # Arguments
  ///
  /// * `crate_name` - The name of the crate to get the path for.
  ///
  /// * `known_re_exporting_crates` - A list of known crates that re-export the proc macro.
  /// This is useful for monorepos like bevy where the user typically only depends on the main bevy crate but uses
  /// proc macros from subcrates like `bevy_ecs`.
  /// If a direct dependency exists, it is preferred over a re-exporting crate.
  pub fn try_resolve_crate_path(&self, crate_name: &str, known_re_exporting_crates: &[&KnownReExportingCrate<'_>]) -> Result<syn::Path, TryResolveCratePathError> {
    info!(
      "Trying to get the path for: {}",
      crate_name
    );

    // Check if we have a direct dependency.
    let directly_mapped_crate_name = self.resolved_dependencies.0.get(crate_name);
    if let Some(directly_mapped_crate_name) = directly_mapped_crate_name {
      let directly_mapped_crate_name = match directly_mapped_crate_name {
        Ok(crate_name) => crate_name,
        Err(err) => {
          return Err(*err.clone());
        },
      };
      return Ok(syn::parse_str::<syn::Path>(directly_mapped_crate_name).unwrap());
    }

    for known_re_exporting_crate in known_re_exporting_crates {

    }


    /// Gets the absolute module path for a crate from a supplied dependencies section.
    ///
    /// Crates that had their module path remapped are also supported.
    ///
    /// For the normal crate case:
    ///
    /// ```toml
    /// [dependencies]
    /// original-crate-name = "0.1"
    /// ```
    ///
    /// The function would return `Some("original-crate-name")` for the `Item` above.
    ///
    /// For the remapped crate case:
    ///
    /// ```toml
    /// [dependencies]
    /// renamed-crate-name = { version = "0.1", package = "original-crate-name" }
    /// ```
    ///
    /// The function would return `Some("renamed-crate-name")` for the `Item` above.
    ///
    fn get_absolute_module_path_for_crate<'a>(
      dependencies_section: &'a Item,
      crate_name: &str, known_re_exporting_crates: &[&KnownReExportingCrate<'_>]
    ) -> Option<&'a str> {
      let dependencies_table = dependencies_section
        .as_table()
        .expect("The dependencies section in the Cargo.toml is not a table");

      let mut found_re_exported_path: Option<syn::Path> = None;

      // Iterate over all dependencies
      for (name, dependency_item) in dependencies_table
      {
        // Get the actual dependency name whether it is remapped or not
        let dependency_crate_name = *dependency_item.get("package").map(|package_field| {
          package_field
            .as_str()
            .expect("The package name in the Cargo.toml is not a string")
        }).get_or_insert(name);

        // Check if we have a direct dependency
        if dependency_crate_name == crate_name {
          return Some(name);
        }
        // Check with the known re-exporting crates
        if found_re_exported_path.is_none() {
          for known_re_exporting_crate in known_re_exporting_crates {
            if known_re_exporting_crate.re_exporting_crate_name == dependency_crate_name {
              // we found a re-exporting crate in the dependencies section
              found_re_exported_path = known_re_exporting_crate.crate_re_exporting_policy.get_re_exported_crate_path(crate_name);
            }
          }
        }
      }
      todo!()
    }

    let find_in_dependencies_section = |dependencies_section: &Item| -> Option<syn::Path> {
      let package = ;

      let mut path = Self::parse_str::<syn::Path>(package);
      let module = crate_name.strip_prefix("beaver_")?;
      path.segments.push(Self::parse_str(module));
      Some(path)
    };

    let deps = self.cargo_manifest.get("dependencies");
    let deps_dev = self.cargo_manifest.get("dev-dependencies");
    let deps_build = self.cargo_manifest.get("build-dependencies");

    // Try to find the subcrate in the dependencies section first, then in the dev-dependencies section
    let ret = deps
      .and_then(find_in_dependencies_section)
      .or_else(|| deps_dev.and_then(find_in_dependencies_section));
    info!(
      "Computed path: {:?} for {}",
      ret.clone().map(pretty_format_syn_path),
      crate_name
    );
    ret
  }

  /// Returns the path for the crate with the given name.
  pub fn get_path(&self, name: &str) -> syn::Path {
    let syn_path = syn::parse_str::<syn::Path>(name).unwrap();
    let copy = syn_path.clone();
    let appended = syn_path.segments
    self
      .try_resolve_crate_path(name)
      .unwrap_or_else(|| Self::parse_str(name))
  }

  /// Attempt to parse the provided [path](str) as a [syntax tree node](syn::parse::Parse)
  pub fn try_parse_str<T: syn::parse::Parse>(path: &str) -> Option<T> {
    syn::parse(path.parse::<TokenStream>().ok()?).ok()
  }

  /// Attempt to parse provided [path](str) as a [syntax tree node](syn::parse::Parse).
  ///
  /// # Panics
  ///
  /// Will panic if the path is not able to be parsed. For a non-panicking option, see [`try_parse_str`]
  ///
  /// [`try_parse_str`]: Self::try_parse_str
  pub fn parse_str<T: syn::parse::Parse>(path: &str) -> T {
    Self::try_parse_str(path).unwrap()
  }

  /// Attempt to get a subcrate [path](syn::Path) under Bevy by [name](str)
  pub fn get_subcrate(&self, subcrate: &str) -> Option<syn::Path> {
    self
      .try_resolve_crate_path(MAIN_PACKAGE_NAME)
      .map(|bevy_path| {
        let mut segments = bevy_path.segments;
        segments.push(CargoManifest::parse_str(subcrate));
        syn::Path {
          leading_colon: None,
          segments,
        }
      })
      .or_else(|| self.try_resolve_crate_path(&format!("bevy_{subcrate}")))
  }
}
