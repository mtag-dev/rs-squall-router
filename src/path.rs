use regex::Regex;
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Param {
    pub index: usize,
    pub validator: Option<Regex>,
}

#[derive(Debug)]
pub struct Path<'a> {
    pub origin: &'a str,
    pub octets: Vec<Cow<'a, str>>,
    pub params_names: Vec<Cow<'a, str>>,
    pub params_values: Vec<Param>,
    pub params_len: usize,
}

pub struct PathParser {
    validators: HashMap<String, Regex>,
}

impl<'a> PathParser {
    /// Returns a path parser backed with provided validators
    ///
    /// # Examples
    ///
    /// ```
    /// use squall_core::path::PathParser;
    ///
    /// let parser = PathParser::new();
    /// ```
    pub fn new() -> PathParser {
        PathParser {
            validators: HashMap::new(),
        }
    }

    fn is_valid(&self, path: &str) -> bool {
        Regex::new(r"^[/a-zA-Z0-9_:{}%\-~!&'*+,;=@]+$")
            .unwrap()
            .is_match(path)
    }

    /// Returns trimmed path without start/end slashes/Regex artifacts
    ///
    /// # Arguments
    ///
    /// * `path` - original path value
    fn normalized(&self, path: &'a str) -> &'a str {
        path.trim_start_matches("^")
            .trim_start_matches("/")
            .trim_end_matches("$")
            .trim_end_matches("/")
    }

    /// Returns a path split by octets. Any complete dynamic octet replaced by asterisk
    /// If octet is partially dynamic returns an error.
    /// "api/v1/user/{user_id}" <- Valid
    /// "api/v1/user/ID-{user_id}" <- Will cause an error
    ///
    /// Sometimes, part of code is better than words )
    /// `assert_eq(self.get_octets("api/v1/user/{user_id}"), vec!["api", "v1", "user", "*"]))`
    ///
    /// # Arguments
    ///
    /// * `path` - Normalized(trimmed) path
    ///
    fn get_octets(&self, path: &str) -> Result<Vec<Cow<str>>, String> {
        let patterns = [Regex::new(r"\{([^}]*)\}").unwrap()];
        let mut normalized = path.to_string();
        for pattern in patterns {
            normalized = pattern
                .replace_all(normalized.as_str(), "*")
                .as_ref()
                .to_string();
        }

        let mut result = Vec::new();
        let mut errors = Vec::new();

        for i in normalized.split("/") {
            if i.len() == 0 {
                continue;
            }

            let octet = match i {
                val if val == "*" => val,
                val if val.contains("*") => {
                    errors.push(val);
                    val
                }
                val => val,
            };

            result.push(Cow::from(octet.to_owned()));
        }
        if errors.is_empty() {
            Ok(result)
        } else {
            Err("Invalid path".to_string())
        }
    }

    /// Returns a vector of dynamic parameters objects (Param)
    /// In case if parameter validator not found in PathParser.validators, will cause an error.
    /// If no validator specified it will be processed as str.
    ///
    /// # Arguments
    ///
    /// * `path` - Normalized(trimmed) path
    ///
    fn get_params(&self, path: &str) -> Result<(Vec<Cow<str>>, Vec<Param>), String> {
        let param_pattern =
            Regex::new(r"^\{([a-zA-Z_][a-zA-Z0-9_]*)(:[a-zA-Z_][a-zA-Z0-9_]*)?\}$").unwrap();
        let mut names = Vec::new();
        let mut matched = Vec::new();

        for (index, octet) in path.split("/").enumerate() {
            if let Some(cap) = param_pattern.captures(octet) {
                let name = cap.get(1).unwrap().as_str();
                let value = match cap.get(2) {
                    Some(v) => {
                        let validator = v.as_str().trim_start_matches(":");
                        if validator == "str" {
                            None
                        } else {
                            if let Some(v) = self.validators.get(validator) {
                                Some(v.to_owned())
                            } else {
                                None
                            }
                        }
                    }
                    None => None,
                };
                names.push(Cow::from(name.to_owned()));
                matched.push(Param {
                    index,
                    validator: value,
                })
            }
        }

        return Ok((names, matched));
    }

    /// Adds new validator
    ///
    /// # Arguments
    ///
    /// * `validator` - String validator identifier
    /// * `regex` - String Regex pattern for compiling validator
    ///
    /// # Examples
    ///
    /// ```
    /// use squall_core::path::PathParser;
    ///
    /// let mut parser = PathParser::new();
    /// parser.add_validator("int".to_string(), r"[0-9]+".to_string());
    /// ```
    pub fn add_validator(&mut self, validator: String, regex: String) -> Result<(), String> {
        // Adds new dynamic octet type validator
        match Regex::new(regex.as_str()) {
            Ok(v) => {
                self.validators.insert(validator, v);
                Ok(())
            }
            Err(e) => Err(e.to_string()),
        }
    }

    /// Main method
    ///
    /// Explanation:
    /// assert_eq!(
    ///     parser.parse("/route/aaa/{num}/bbb/{num2:str}/ccc/{num3:int}"),
    ///     Path {
    ///         origin: "/route/aaa/{num}/bbb/{num2:str}/ccc/{num3:int}",
    ///         octets: ["route", "aaa", "*", "bbb", "*", "ccc", "*"],
    ///         params_names: vec![]
    ///         params: [
    ///             Param { index: 2, name: "num", validator: None },
    ///             Param { index: 4, name: "num2", validator: None },
    ///             Param { index: 6, name: "num3", validator: Some(Regex::new("[0-9]+").unwrap()) }
    ///         ]
    ///     }
    /// )
    pub fn parse(&'a self, path: &'a str) -> Result<Path<'a>, String> {
        if self.is_valid(path) {
            let normalized = self.normalized(path);
            let octets = match self.get_octets(normalized) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            let (params_names, params_values) = match self.get_params(normalized) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            let params_len = params_names.len();
            return Ok(Path {
                origin: path,
                octets,
                params_names,
                params_values,
                params_len: params_len,
            });
        }
        Err("Path processing error".to_string())
    }
}
