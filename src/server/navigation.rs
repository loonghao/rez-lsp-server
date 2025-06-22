//! Navigation features for the LSP server (Go to Definition, Find References, etc.).

use crate::core::{traits::PackageDiscovery, types::Package, Result};
use crate::discovery::PackageDiscoveryImpl;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::lsp_types::*;

/// Handles navigation requests like Go to Definition and Find References.
pub struct NavigationHandler {
    /// Package discovery service
    package_discovery: Arc<RwLock<Option<PackageDiscoveryImpl>>>,
}

impl NavigationHandler {
    /// Create a new navigation handler.
    pub fn new(package_discovery: Arc<RwLock<Option<PackageDiscoveryImpl>>>) -> Self {
        Self { package_discovery }
    }

    /// Handle "Go to Definition" requests.
    pub async fn handle_goto_definition(
        &self,
        params: &GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = &params.text_document_position_params.position;

        // Only handle package.py files
        if !uri.path().ends_with("package.py") {
            return Ok(None);
        }

        // Extract the word at the cursor position
        if let Some(word) = self.extract_word_at_position(uri, position).await? {
            // Check if it's a package reference
            if let Some(package) = self.find_package_definition(&word).await? {
                let location = self.package_to_location(&package)?;
                return Ok(Some(GotoDefinitionResponse::Scalar(location)));
            }
        }

        Ok(None)
    }

    /// Handle "Find References" requests.
    pub async fn handle_find_references(
        &self,
        params: &ReferenceParams,
    ) -> Result<Option<Vec<Location>>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = &params.text_document_position.position;

        // Only handle package.py files
        if !uri.path().ends_with("package.py") {
            return Ok(None);
        }

        // Extract the word at the cursor position
        if let Some(word) = self.extract_word_at_position(uri, position).await? {
            // Find all references to this package
            let references = self.find_package_references(&word).await?;
            if !references.is_empty() {
                return Ok(Some(references));
            }
        }

        Ok(None)
    }

    /// Handle "Document Symbols" requests.
    pub async fn handle_document_symbols(
        &self,
        params: &DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = &params.text_document.uri;

        // Only handle package.py files
        if !uri.path().ends_with("package.py") {
            return Ok(None);
        }

        // Parse the document and extract symbols
        let symbols = self.extract_document_symbols(uri).await?;
        if !symbols.is_empty() {
            return Ok(Some(DocumentSymbolResponse::Nested(symbols)));
        }

        Ok(None)
    }

    /// Handle "Workspace Symbols" requests.
    pub async fn handle_workspace_symbols(
        &self,
        params: &WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        let query = &params.query;

        // Search for packages matching the query
        let symbols = self.find_workspace_symbols(query).await?;
        if !symbols.is_empty() {
            return Ok(Some(symbols));
        }

        Ok(None)
    }

    /// Extract the word at a specific position in a document.
    async fn extract_word_at_position(
        &self,
        _uri: &Url,
        _position: &Position,
    ) -> Result<Option<String>> {
        // TODO: Implement actual text parsing
        // For now, return a placeholder
        Ok(Some("example_package".to_string()))
    }

    /// Find the definition of a package.
    async fn find_package_definition(&self, package_name: &str) -> Result<Option<Package>> {
        let discovery = self.package_discovery.read().await;
        if let Some(discovery) = discovery.as_ref() {
            let packages = discovery.find_packages(package_name).await?;
            // Return the latest version
            Ok(packages.into_iter().max_by_key(|p| p.version.clone()))
        } else {
            Ok(None)
        }
    }

    /// Find all references to a package.
    async fn find_package_references(&self, _package_name: &str) -> Result<Vec<Location>> {
        // TODO: Implement reference finding
        // This would involve scanning all package.py files for references
        Ok(Vec::new())
    }

    /// Extract document symbols from a package.py file.
    async fn extract_document_symbols(&self, _uri: &Url) -> Result<Vec<DocumentSymbol>> {
        // TODO: Implement symbol extraction
        // This would parse the package.py file and extract:
        // - Package name
        // - Version
        // - Requirements
        // - Tools
        // - Functions (build, commands, etc.)

        #[allow(deprecated)]
        let symbols = vec![
            DocumentSymbol {
                name: "name".to_string(),
                detail: Some("Package name".to_string()),
                kind: SymbolKind::VARIABLE,
                tags: None,
                deprecated: None,

                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 20,
                    },
                },
                selection_range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 4,
                    },
                },
                children: None,
            },
            DocumentSymbol {
                name: "version".to_string(),
                detail: Some("Package version".to_string()),
                kind: SymbolKind::VARIABLE,
                tags: None,
                deprecated: None,
                range: Range {
                    start: Position {
                        line: 1,
                        character: 0,
                    },
                    end: Position {
                        line: 1,
                        character: 20,
                    },
                },
                selection_range: Range {
                    start: Position {
                        line: 1,
                        character: 0,
                    },
                    end: Position {
                        line: 1,
                        character: 7,
                    },
                },
                children: None,
            },
        ];

        Ok(symbols)
    }

    /// Find workspace symbols matching a query.
    async fn find_workspace_symbols(&self, query: &str) -> Result<Vec<SymbolInformation>> {
        let discovery = self.package_discovery.read().await;
        if let Some(discovery) = discovery.as_ref() {
            let packages = discovery.find_packages(query).await?;
            let mut symbols = Vec::new();

            for package in packages {
                if let Ok(location) = self.package_to_location(&package) {
                    #[allow(deprecated)]
                    let symbol = SymbolInformation {
                        name: package.name.clone(),
                        kind: SymbolKind::PACKAGE,
                        tags: None,
                        deprecated: None,
                        location,
                        container_name: None,
                    };
                    symbols.push(symbol);
                }
            }

            Ok(symbols)
        } else {
            Ok(Vec::new())
        }
    }

    /// Convert a package to a location.
    fn package_to_location(&self, package: &Package) -> Result<Location> {
        let uri = Url::from_file_path(&package.path).map_err(|_| {
            crate::core::Error::InvalidPath(package.path.to_string_lossy().to_string())
        })?;

        Ok(Location {
            uri,
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 0,
                },
            },
        })
    }
}

/// Extract package references from text content.
#[allow(dead_code)]
pub fn extract_package_references(content: &str) -> Vec<PackageReference> {
    let mut references = Vec::new();

    // Simple regex-based extraction for now
    // TODO: Use proper Python AST parsing
    for (line_num, line) in content.lines().enumerate() {
        if line.trim().starts_with("requires") {
            // Extract package names from requires list
            if let Some(start) = line.find('[') {
                if let Some(end) = line.find(']') {
                    let requires_content = &line[start + 1..end];
                    for requirement in requires_content.split(',') {
                        let requirement = requirement.trim().trim_matches('"').trim_matches('\'');
                        if !requirement.is_empty() {
                            // Extract package name (before version constraints)
                            let package_name = requirement
                                .split(&['<', '>', '=', '!'][..])
                                .next()
                                .unwrap_or(requirement)
                                .trim();

                            if !package_name.is_empty() {
                                references.push(PackageReference {
                                    package_name: package_name.to_string(),
                                    line: line_num as u32,
                                    column: line.find(package_name).unwrap_or(0) as u32,
                                    length: package_name.len() as u32,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    references
}

/// A reference to a package in source code.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PackageReference {
    /// Name of the referenced package
    pub package_name: String,
    /// Line number (0-based)
    pub line: u32,
    /// Column number (0-based)
    pub column: u32,
    /// Length of the reference
    pub length: u32,
}

#[allow(dead_code)]
impl PackageReference {
    /// Convert to LSP Range.
    pub fn to_range(&self) -> Range {
        Range {
            start: Position {
                line: self.line,
                character: self.column,
            },
            end: Position {
                line: self.line,
                character: self.column + self.length,
            },
        }
    }

    /// Convert to LSP Location.
    pub fn to_location(&self, uri: Url) -> Location {
        Location {
            uri,
            range: self.to_range(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_package_references() {
        let content = r#"
name = "test_package"
version = "1.0.0"
requires = ["python>=3.7", "maya>=2020", "houdini"]
"#;

        let references = extract_package_references(content);
        assert_eq!(references.len(), 3);

        assert_eq!(references[0].package_name, "python");
        assert_eq!(references[1].package_name, "maya");
        assert_eq!(references[2].package_name, "houdini");
    }

    #[test]
    fn test_package_reference_to_range() {
        let reference = PackageReference {
            package_name: "test".to_string(),
            line: 5,
            column: 10,
            length: 4,
        };

        let range = reference.to_range();
        assert_eq!(range.start.line, 5);
        assert_eq!(range.start.character, 10);
        assert_eq!(range.end.line, 5);
        assert_eq!(range.end.character, 14);
    }

    #[tokio::test]
    async fn test_navigation_handler_creation() {
        use std::sync::Arc;
        use tokio::sync::RwLock;

        let package_discovery = Arc::new(RwLock::new(None));
        let _handler = NavigationHandler::new(package_discovery);

        // Basic smoke test - handler created successfully
    }
}
