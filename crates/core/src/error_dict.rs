use crate::types::ErrorExplanation;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Global dictionary of known Sui errors mapped to plain-English explanations.
pub struct ErrorDictionary;

impl ErrorDictionary {
    /// Look up an error string exactly, or try to fuzzy-match known patterns.
    pub fn lookup(error_string: &str) -> Option<ErrorExplanation> {
        let dict = get_dictionary();

        // 1. Case-insensitive exact match
        for (pattern, explanation) in dict.iter() {
            if error_string.eq_ignore_ascii_case(pattern) {
                return Some(explanation.clone());
            }
        }

        if error_string.contains("MoveAbort") {
            return Self::fuzzy_match_move_abort(error_string);
        }

        // 3. Case-insensitive fuzzy match without allocating a lowercase string
        for (pattern, explanation) in dict.iter() {
            if case_insensitive_contains(error_string, pattern) {
                return Some(explanation.clone());
            }
        }

        None
    }

    fn fuzzy_match_move_abort(error_string: &str) -> Option<ErrorExplanation> {
        let code = error_string
            .rfind(')')
            .and_then(|last_close| {
                error_string[..last_close]
                    .rfind(',')
                    .map(|last_comma| (last_comma, last_close))
            })
            .and_then(|(last_comma, last_close)| {
                error_string[last_comma + 1..last_close]
                    .trim()
                    .parse::<u64>()
                    .ok()
            });

        if let Some(code) = code {
            let prefix = "name: Identifier(\"";
            let module_name = if let Some(start_idx) = error_string.find(prefix) {
                let start = start_idx + prefix.len();
                if let Some(end_idx) = error_string[start..].find("\")") {
                    &error_string[start..start + end_idx]
                } else {
                    ""
                }
            } else {
                ""
            };

            if !module_name.is_empty() {
                return match (module_name, code) {
                    ("coin", 2) => Some(ErrorExplanation {
                        title: "Insufficient Coin Balance (Code 2)".into(),
                        plain_english: "The coin object provided doesn't have enough balance.".into(),
                        likely_causes: vec!["You are trying to pay for something with a coin that doesn't hold enough SUI.".into()],
                        suggested_fixes: vec!["Merge your coin objects or use a different one.".into()],
                    }),
                    ("object", 1) | ("transfer", 1) => Some(ErrorExplanation {
                        title: "Not Authorized / Invalid Owner (Code 1)".into(),
                        plain_english: "You do not have permission to perform this action.".into(),
                        likely_causes: vec![
                            "You are trying to mutate an object you do not own.".into(),
                            "You forgot to pass a required capability object.".into(),
                        ],
                        suggested_fixes: vec![
                            "Check the current owner of the object using `suiscope inspect <id>`.".into(),
                            "Ensure your wallet is the active one in `sui client active-address`.".into(),
                        ],
                    }),
                    (_, 0) => Some(ErrorExplanation {
                        title: "Assertion Failed (Code 0)".into(),
                        plain_english: "A basic assertion failed in the Move contract.".into(),
                        likely_causes: vec!["The contract expected a condition to be true, but it was false.".into()],
                        suggested_fixes: vec!["Check the contract source code for `assert!(..., 0)`.".into()],
                    }),
                    _ => Some(ErrorExplanation {
                        title: format!("Move Abort in '{}' (Code {})", module_name, code),
                        plain_english: "The smart contract aborted execution intentionally.".into(),
                        likely_causes: vec!["A requirement in the contract was not met.".into()],
                        suggested_fixes: vec![
                            format!("Look up the `{}` module source code and find the `abort` or `assert!` that throws code {}.", module_name, code)
                        ],
                    }),
                };
            } else {
                return match code {
                    0 => Some(ErrorExplanation {
                        title: "Assertion Failed (Code 0)".into(),
                        plain_english: "A basic assertion failed in the Move contract.".into(),
                        likely_causes: vec!["The contract expected a condition to be true, but it was false.".into()],
                        suggested_fixes: vec!["Check the contract source code for `assert!(..., 0)`.".into()],
                    }),
                    1 => Some(ErrorExplanation {
                        title: "Not Authorized / Invalid Owner (Code 1)".into(),
                        plain_english: "You do not have permission to perform this action.".into(),
                        likely_causes: vec![
                            "You are trying to mutate an object you do not own.".into(),
                            "You forgot to pass a required capability object.".into(),
                        ],
                        suggested_fixes: vec![
                            "Check the current owner of the object using `suiscope inspect <id>`.".into(),
                            "Ensure your wallet is the active one in `sui client active-address`.".into(),
                        ],
                    }),
                    2 => Some(ErrorExplanation {
                        title: "Invalid Coin / Insufficient Balance (Code 2)".into(),
                        plain_english: "The coin object provided doesn't have enough balance or is the wrong type.".into(),
                        likely_causes: vec!["You are trying to pay for something with a coin that doesn't hold enough SUI.".into()],
                        suggested_fixes: vec!["Merge your coin objects or use a different one.".into()],
                    }),
                    _ => Some(ErrorExplanation {
                        title: format!("Move Abort (Code {})", code),
                        plain_english: "The smart contract aborted execution intentionally.".into(),
                        likely_causes: vec!["A requirement in the contract was not met.".into()],
                        suggested_fixes: vec![
                            "Look up the module source code and find the `abort` or `assert!` that throws this code.".into()
                        ],
                    }),
                };
            }
        }

        Some(ErrorExplanation {
            title: "Move Abort".into(),
            plain_english: "The transaction was aborted by the smart contract.".into(),
            likely_causes: vec!["A condition in the contract failed.".into()],
            suggested_fixes: vec!["Check the exact abort code in the transaction logs.".into()],
        })
    }
}

fn get_dictionary() -> &'static HashMap<&'static str, ErrorExplanation> {
    static DICT: OnceLock<HashMap<&'static str, ErrorExplanation>> = OnceLock::new();
    DICT.get_or_init(|| {
        let mut m = HashMap::new();

        // RPC Errors
        m.insert("-32602", ErrorExplanation {
            title: "Invalid Parameters (-32602)".into(),
            plain_english: "The RPC node rejected your request because the parameters were formatted incorrectly.".into(),
            likely_causes: vec![
                "You passed an invalid Object ID (not 64 hex characters).".into(),
                "Missing required parameters in the JSON-RPC call.".into(),
            ],
            suggested_fixes: vec![
                "Ensure your Object ID starts with '0x' and is exactly 64 characters long.".into(),
            ],
        });

        // Common strings
        m.insert("invalid object reference", ErrorExplanation {
            title: "Invalid Object Reference".into(),
            plain_english: "You tried to use an object that the network doesn't recognise or that you can't access.".into(),
            likely_causes: vec![
                "The object doesn't exist on this network (e.g., it's a testnet object but you're on mainnet).".into(),
                "The object ID is malformed.".into(),
            ],
            suggested_fixes: vec![
                "Verify your current network with `suiscope get-config` or check your .suiscope/config.toml.".into(),
                "Ensure you copied the entire 64-character Object ID.".into(),
            ],
        });

        m.insert("version conflict", ErrorExplanation {
            title: "Version Conflict / Sequence Number Error".into(),
            plain_english: "You are trying to use an old version of an object.".into(),
            likely_causes: vec![
                "Another transaction modified this object recently, incrementing its version number.".into(),
                "You are sending multiple transactions at the same time using the same object.".into(),
            ],
            suggested_fixes: vec![
                "Fetch the latest object state using `suiscope inspect <id>` to get the current version.".into(),
                "Avoid sending concurrent transactions that mutate the same object.".into(),
            ],
        });

        m.insert("package not found", ErrorExplanation {
            title: "Package Not Found".into(),
            plain_english: "The smart contract package you are trying to call doesn't exist.".into(),
            likely_causes: vec![
                "You haven't published the package yet.".into(),
                "You are connected to the wrong network (e.g. testnet instead of localnet).".into(),
            ],
            suggested_fixes: vec![
                "Run `suiscope publish` to deploy the package.".into(),
                "Check your configured network.".into(),
            ],
        });

        m
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let expl = ErrorDictionary::lookup("-32602").unwrap();
        assert_eq!(expl.title, "Invalid Parameters (-32602)");
    }

    #[test]
    fn test_fuzzy_match_string() {
        // "Rpc Error: invalid object reference 0x123..."
        let expl = ErrorDictionary::lookup("Rpc Error: invalid object reference 0x123...").unwrap();
        assert_eq!(expl.title, "Invalid Object Reference");
    }

    #[test]
    fn test_fuzzy_match_move_abort() {
        let err = "MoveAbort(MoveLocation { module: ModuleId { address: 000...2, name: Identifier(\"object\") }, function: 1, instruction: 10, function_name: Some(\"new\") }, 1)";
        let expl = ErrorDictionary::lookup(err).unwrap();
        assert_eq!(expl.title, "Not Authorized / Invalid Owner (Code 1)");
    }

    #[test]
    fn test_fuzzy_match_move_abort_unknown_code() {
        let err = "MoveAbort(MoveLocation { module: ModuleId { address: 000...2, name: Identifier(\"object\") }, function: 1, instruction: 10, function_name: Some(\"new\") }, 999)";
        let expl = ErrorDictionary::lookup(err).unwrap();
        assert_eq!(expl.title, "Move Abort in 'object' (Code 999)");
    }
}

/// Case-insensitive substring search that avoids allocating new lowercase strings.
fn case_insensitive_contains(haystack: &str, needle: &str) -> bool {
    let haystack_bytes = haystack.as_bytes();
    let needle_bytes = needle.as_bytes();
    if needle_bytes.is_empty() {
        return true;
    }
    if haystack_bytes.len() < needle_bytes.len() {
        return false;
    }
    
    haystack_bytes.windows(needle_bytes.len()).any(|window| {
        window.iter().zip(needle_bytes).all(|(&h, &n)| {
            h.eq_ignore_ascii_case(&n)
        })
    })
}
