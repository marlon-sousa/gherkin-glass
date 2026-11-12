//! Entity: where a file is in the site, relative to its root.

use derive_more::Display;
/// The names of the folders the file is in, from the outermost, then the file's name, each as
/// written and joined by `/`.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Display)]
pub(crate) struct SiteAddress {
    address: String,
}

impl SiteAddress {
    /// The file named `file` in `folders`, from the outermost.
    pub(crate) fn of<Folder: AsRef<str>>(folders: &[Folder], file: &str) -> SiteAddress {
        SiteAddress {
            address: folders
                .iter()
                .map(AsRef::as_ref)
                .chain([file])
                .collect::<Vec<_>>()
                .join("/"),
        }
    }

    /// The file named `file` in the site's root.
    pub(crate) fn at_root(file: &str) -> SiteAddress {
        SiteAddress {
            address: file.to_string(),
        }
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.address
    }

    /// The folders the file is in, from the outermost.
    pub(crate) fn folders(&self) -> Vec<&str> {
        self.address
            .rsplit_once('/')
            .map(|(folders, _)| folders.split('/').collect())
            .unwrap_or_default()
    }

    /// The file's own name.
    pub(crate) fn file(&self) -> &str {
        self.address
            .rsplit_once('/')
            .map_or(self.address.as_str(), |(_, file)| file)
    }

    /// The relative address from the file back to the site's root, such as `../` for a file
    /// one folder down, and empty for a file at the root.
    pub(crate) fn root(&self) -> String {
        "../".repeat(self.address.matches('/').count())
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::at_the_root(&[], "a.html", "a.html", "")]
    #[case::two_folders_down(&["billing", "refunds"], "a.html", "billing/refunds/a.html", "../../")]
    fn an_address_is_its_folders_then_its_file_and_its_root_one_step_up_for_each_folder(
        #[case] folders: &[&str],
        #[case] file: &str,
        #[case] written: &str,
        #[case] root: &str,
    ) {
        let address = SiteAddress::of(folders, file);
        assert_eq!(address.as_str(), written);
        assert_eq!(address.folders(), folders);
        assert_eq!(address.file(), file);
        assert_eq!(address.root(), root);
    }
}
