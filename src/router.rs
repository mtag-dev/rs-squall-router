use crate::path::{Param, PathParser};
use firestorm::{profile_fn, profile_method};
use rustc_hash::FxHashMap;
use std::str;

#[derive(Debug)]
struct Handler {
    handler: i32,
    method: String,
    params_names: Vec<String>,
    params_values: Vec<Param>,
    params_len: usize,
}

#[derive(Default, Debug)]
struct Database {
    children: FxHashMap<String, Database>,
    handlers: Vec<Handler>,
}

#[inline]
fn get_path_handlers<'a>(
    database_root: &'a Vec<Database>,
    path: &'a str,
    octets_len: usize,
    allow_empty_octets: bool,
) -> Option<&'a Vec<Handler>> {
    profile_fn!(get_path_handlers);
    let mut is_first_octet = true;

    if let Some(mut database) = database_root.get(octets_len) {
        for octet in path.as_bytes().split(|b| b == &b'/') {
            if octet.is_empty() {
                if is_first_octet {
                    continue;
                } else if allow_empty_octets {
                    continue;
                }
            }

            is_first_octet = false;

            let str_octet = unsafe { str::from_utf8_unchecked(octet) };
            match database.children.get(str_octet) {
                Some(v) => database = v,
                None => match database.children.get("*") {
                    Some(dynamic) => database = dynamic,
                    None => return None,
                },
            }
        }
        return Some(&database.handlers);
    }
    None
}

pub struct SquallRouter {
    dynamic_db: Vec<Database>,
    dynamic_db_size: usize,
    static_db: FxHashMap<String, Vec<Handler>>,
    locations_db: Vec<(String, Vec<Handler>)>,
    path_parser: PathParser,
    ingore_trailing_slashes: bool,
}

impl SquallRouter {
    pub fn new() -> Self {
        SquallRouter {
            dynamic_db: Vec::new(),
            dynamic_db_size: 0,
            static_db: FxHashMap::default(),
            locations_db: Vec::new(),
            path_parser: PathParser::new(),
            ingore_trailing_slashes: false,
        }
    }

    /// Enable ignore trailing slashes mode
    ///
    /// # Examples
    ///
    /// ```
    /// use squall_router::SquallRouter;
    ///
    /// let mut router = SquallRouter::new();
    /// router.set_ignore_trailing_slashes();
    /// ```
    pub fn set_ignore_trailing_slashes(&mut self) {
        self.ingore_trailing_slashes = true;
        self.path_parser.set_ignore_trailing_slashes();
    }

    /// Adds new validation option for dynamic parameters.
    ///
    /// # Arguments
    ///
    /// * `alias` - String validator alias
    /// * `regex` - String Regex pattern for compiling validator
    ///
    /// # Examples
    ///
    /// ```
    /// use squall_router::SquallRouter;
    ///
    /// let mut router = SquallRouter::new();
    /// router.add_validator("int".to_string(), r"[0-9]+".to_string());
    /// ```
    pub fn add_validator(&mut self, alias: String, regex: String) -> Result<(), String> {
        self.path_parser.add_validator(alias, regex)
    }

    /// Adds new route.
    ///
    /// # Arguments
    ///
    /// * `method` - Method name. At the moment any String.
    ///              U can use it also for WS endpoints registration, for instance `"WS".to_string()`
    /// * `path` - String path string.
    /// * `handler` - Handler function identifier.
    ///
    /// # Examples
    ///
    /// Simple routes registration
    /// ```
    /// use squall_router::SquallRouter;
    ///
    /// let mut router = SquallRouter::new();
    /// router.add_route("GET".to_string(), "/api/users".to_string(), 0);
    /// router.add_route("GET".to_string(), "/api/user/{user_id}".to_string(), 1);
    /// ```
    ///
    /// Extra route parameters validation
    /// ```
    /// use squall_router::SquallRouter;
    ///
    /// let mut router = SquallRouter::new();
    /// router.add_validator("int".to_string(), r"[0-9]+".to_string());
    /// router.add_route("GET".to_string(), "/api/user/{user_id:int}".to_string(), 0);
    /// ```
    pub fn add_route(&mut self, method: String, path: String, handler: i32) -> Result<(), String> {
        let _path = match self.ingore_trailing_slashes {
            true => path.trim_end_matches("/").to_string(),
            false => path,
        };

        match self.path_parser.parse(_path.as_str()) {
            Ok(parsed) => {
                let params_names = parsed
                    .params_names
                    .iter()
                    .map(|v| v.as_ref().to_owned())
                    .collect();

                let handler = Handler {
                    handler,
                    method,
                    params_names,
                    params_values: parsed.params_values,
                    params_len: parsed.params_len,
                };

                // If path completely static, just add to static DB
                if parsed.octets.iter().all(|i| i != "*") {
                    self.static_db
                        .entry(_path)
                        .or_insert_with(Vec::default)
                        .push(handler);
                    return Ok(());
                }

                // resize dynamic DB if needed
                let depth = parsed.octets.len();

                if depth + 1 > self.dynamic_db.len() {
                    self.dynamic_db.resize_with(depth + 1, Database::default);
                    self.dynamic_db_size = self.dynamic_db.len();
                }

                // iterate through the path octets and build database tree
                let mut node = &mut self.dynamic_db[depth];
                for subkey in parsed.octets {
                    node = node
                        .children
                        .entry(subkey.to_string())
                        .or_insert_with(Database::default);
                }

                node.handlers.push(handler);
                return Ok(());
            }
            Err(e) => Err(e),
        }
    }

    /// Adds new location for prefixed requests handling
    ///
    /// # Arguments
    ///
    /// * `method` - Method name. At the moment any String.
    ///              U can use it also for WS endpoints registration, for instance `"WS".to_string()`
    /// * `path` - String path string.
    /// * `handler` - Handler function identifier.
    ///
    /// # Examples
    ///
    /// ```
    /// use squall_router::SquallRouter;
    ///
    /// let mut router = SquallRouter::new();
    /// router.add_location("GET".to_string(), "/assets".to_string(), 0);
    /// ```
    pub fn add_location(&mut self, method: String, path: String, handler: i32) -> () {
        if let Ok(parsed) = self.path_parser.parse(path.as_str()) {
            let handler = Handler {
                handler,
                method,
                params_names: parsed
                    .params_names
                    .iter()
                    .map(|v| v.as_ref().to_owned())
                    .collect(),
                params_values: parsed.params_values,
                params_len: parsed.params_len,
            };

            for loc in self.locations_db.iter_mut() {
                if loc.0 == path {
                    loc.1.push(handler);
                    return;
                }
            }
            self.locations_db.push((path, vec![handler]));
            self.locations_db.sort_by(|a, b| b.0.cmp(&a.0));
        }
    }

    /// Get handler identifier, param names and values for given method/path.
    ///
    /// Resolving order:
    /// - Static routes
    /// - Dynamic routes
    /// - Locations
    ///
    /// # Arguments
    ///
    /// * `method` - HTTP Method name.
    /// * `path` - Request path.
    ///
    /// # Examples
    ///
    /// ```
    /// use squall_router::SquallRouter;
    ///
    /// let mut router = SquallRouter::new();
    /// router.add_route("GET".to_string(), "/user/{user_id}".to_string(), 0);
    ///
    /// let (handler_id, params) = router.resolve("GET", "/user/123").unwrap();
    /// assert_eq!(handler_id, 0);
    /// assert_eq!(params, vec![("user_id", "123")]);
    /// ```
    #[inline]
    pub fn resolve<'a>(
        &'a self,
        method: &str,
        path: &'a str,
    ) -> Option<(i32, Vec<(&str, &'a str)>)> {
        profile_method!(resolve);

        let _path = match self.ingore_trailing_slashes {
            true => path.trim_end_matches("/"),
            false => path,
        };

        if let Some(v) = self.get_static_path_handler(method, _path) {
            return Some(v);
        }

        if let Some(v) = self.get_dynamic_path_handler(method, _path) {
            return Some(v);
        }

        if let Some(v) = self.get_location_handler(method, _path) {
            return Some(v);
        }

        None
    }

    #[inline]
    fn get_static_path_handler<'a>(
        &'a self,
        method: &str,
        path: &'a str,
    ) -> Option<(i32, Vec<(&str, &'a str)>)> {
        profile_method!(get_static_path_handler);

        if let Some(v) = self.static_db.get(path) {
            for handler in v.iter().filter(|v| v.method == method) {
                return Some((handler.handler, vec![]));
            }
        }
        None
    }

    #[inline]
    fn get_dynamic_path_handler<'a>(
        &'a self,
        method: &str,
        path: &'a str,
    ) -> Option<(i32, Vec<(&str, &'a str)>)> {
        profile_method!(get_dynamic_path_handler);

        let mut octets_len = bytecount::count(path.as_bytes(), b'/');
        if self.ingore_trailing_slashes && path.ends_with("/") {
            octets_len -= 1;
        }

        if let Some(handlers) = get_path_handlers(
            &self.dynamic_db,
            path,
            octets_len,
            self.ingore_trailing_slashes,
        ) {
            'outer: for handler in handlers {
                if &handler.method != method {
                    continue;
                }
                // Names processing should be removed from here
                let mut parameters = Vec::with_capacity(handler.params_len);

                for i in 0..handler.params_len {
                    let param = &handler.params_values[i];
                    let value = unsafe {
                        str::from_utf8_unchecked(
                            path.as_bytes()
                                .split(|b| b == &b'/')
                                .skip(param.index + 1)
                                .next()
                                .unwrap(),
                        )
                    };

                    if let Some(v) = &param.validator {
                        if !v.is_match(value) {
                            continue 'outer;
                        }
                    }
                    parameters.push((handler.params_names[i].as_str(), value));
                }
                return Some((handler.handler, parameters));
            }
        }

        None
    }

    #[inline]
    fn get_location_handler<'a>(
        &'a self,
        method: &str,
        path: &'a str,
    ) -> Option<(i32, Vec<(&str, &'a str)>)> {
        profile_method!(get_location_handler);

        for i in &self.locations_db {
            if !path.starts_with(&i.0) {
                continue;
            }

            for handler in &i.1 {
                if &handler.method != method {
                    continue;
                }

                return Some((handler.handler, vec![]));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_no_validators() {
        let mut router = SquallRouter::new();
        router
            .add_route("GET".to_string(), "/name".to_string(), 0)
            .unwrap();
        router
            .add_route("GET".to_string(), "/name/{val}".to_string(), 1)
            .unwrap();
        router
            .add_route("GET".to_string(), "/name/{val}/index.html".to_string(), 2)
            .unwrap();
        router
            .add_route("GET".to_string(), "/{test}/index.html".to_string(), 3)
            .unwrap();

        let result = router.resolve("GET", "/unknown");
        assert!(result.is_none());

        let result = router.resolve("GET", "/name");
        let (handler, params) = result.unwrap();
        assert_eq!(handler, 0);
        assert!(params.is_empty());

        // Ensure filtered by method
        assert!(router.resolve("POST", "/name").is_none());

        let result = router.resolve("GET", "/name/value");
        let (handler, params) = result.unwrap();
        assert_eq!(handler, 1);
        assert_eq!(params, vec![("val", "value")]);

        let result = router.resolve("GET", "/name/value2/index.html");
        let (handler, params) = result.unwrap();
        assert_eq!(handler, 2);
        assert_eq!(params, vec![("val", "value2")]);

        let result = router.resolve("GET", "/test2/index.html");
        let (handler, params) = result.unwrap();
        assert_eq!(handler, 3);
        assert_eq!(params, vec![("test", "test2")]);
    }

    #[test]
    fn test_resolve_with_validators() {
        let mut router = SquallRouter::new();
        router
            .add_validator("int".to_string(), r"^[0-9]+$".to_string())
            .unwrap();
        router
            .add_validator("no_int".to_string(), r"^[^0-9]+$".to_string())
            .unwrap();
        router
            .add_validator("user_id".to_string(), r"^ID-[0-9]+$".to_string())
            .unwrap();

        router
            .add_route("GET".to_string(), "/user/{user:int}".to_string(), 0)
            .unwrap();
        router
            .add_route("GET".to_string(), "/user/{user:user_id}".to_string(), 1)
            .unwrap();
        router
            .add_route(
                "GET".to_string(),
                "/user/{user:int}/index.html".to_string(),
                2,
            )
            .unwrap();
        router
            .add_route(
                "GET".to_string(),
                "/user/{user:no_int}/index.html".to_string(),
                3,
            )
            .unwrap();

        let result = router.resolve("GET", "/user/123");
        let (handler, params) = result.unwrap();
        assert_eq!(handler, 0);
        assert_eq!(params, vec![("user", "123")]);

        let result = router.resolve("GET", "/user/ID-123");
        let (handler, params) = result.unwrap();
        assert_eq!(handler, 1);
        assert_eq!(params, vec![("user", "ID-123")]);

        let result = router.resolve("GET", "/user/123/index.html");
        let (handler, params) = result.unwrap();
        assert_eq!(handler, 2);
        assert_eq!(params, vec![("user", "123")]);

        let result = router.resolve("GET", "/user/john/index.html");
        let (handler, params) = result.unwrap();
        assert_eq!(handler, 3);
        assert_eq!(params, vec![("user", "john")]);
    }

    #[test]
    fn test_wrong_validator() {
        let mut router = SquallRouter::new();

        assert!(router
            .add_validator("int".to_string(), r"^[0-9+$".to_string())
            .is_err());
    }

    #[test]
    fn test_absent_validator() {
        let mut router = SquallRouter::new();

        let route = router.add_route("GET".to_string(), "/{val:int}".to_string(), 0);

        assert!(route.is_err());
    }

    #[test]
    fn test_ignore_trailing_slashes_enabled() {
        let mut router = SquallRouter::new();
        router.set_ignore_trailing_slashes();
        router
            .add_route("GET".to_string(), "/user/{user}/".to_string(), 2)
            .unwrap();

        router
            .add_route("GET".to_string(), "/issue/{issue}".to_string(), 3)
            .unwrap();

        router
            .add_route("GET".to_string(), "/trailing/".to_string(), 4)
            .unwrap();

        router
            .add_route("GET".to_string(), "/notrailing".to_string(), 5)
            .unwrap();

        let result = router.resolve("GET", "/user/john/");
        let (handler, params) = result.unwrap();
        assert_eq!(handler, 2);
        assert_eq!(params, vec![("user", "john")]);

        let result = router.resolve("GET", "/user/john");
        let (handler, params) = result.unwrap();
        assert_eq!(handler, 2);
        assert_eq!(params, vec![("user", "john")]);

        let result = router.resolve("GET", "/issue/test/");
        let (handler, params) = result.unwrap();
        assert_eq!(handler, 3);
        assert_eq!(params, vec![("issue", "test")]);

        let result = router.resolve("GET", "/issue/test");
        let (handler, params) = result.unwrap();
        assert_eq!(handler, 3);
        assert_eq!(params, vec![("issue", "test")]);

        let result = router.resolve("GET", "/trailing/");
        let (handler, params) = result.unwrap();
        assert_eq!(handler, 4);
        assert_eq!(params, vec![]);

        let result = router.resolve("GET", "/trailing");
        let (handler, params) = result.unwrap();
        assert_eq!(handler, 4);
        assert_eq!(params, vec![]);

        let result = router.resolve("GET", "/notrailing/");
        let (handler, params) = result.unwrap();
        assert_eq!(handler, 5);
        assert_eq!(params, vec![]);

        let result = router.resolve("GET", "/notrailing");
        let (handler, params) = result.unwrap();
        assert_eq!(handler, 5);
        assert_eq!(params, vec![]);
    }

    #[test]
    fn test_ignore_trailing_slashes_disabled() {
        let mut router = SquallRouter::new();
        router
            .add_route("GET".to_string(), "/user/{user}/".to_string(), 2)
            .unwrap();

        router
            .add_route("GET".to_string(), "/issue/{issue}".to_string(), 3)
            .unwrap();

        router
            .add_route("GET".to_string(), "/static/".to_string(), 4)
            .unwrap();

        router
            .add_route("GET".to_string(), "/static".to_string(), 5)
            .unwrap();

        let result = router.resolve("GET", "/user/john/");
        let (handler, params) = result.unwrap();
        assert_eq!(handler, 2);
        assert_eq!(params, vec![("user", "john")]);

        let result = router.resolve("GET", "/user/john");
        assert!(result.is_none());

        let result = router.resolve("GET", "/issue/test/");
        assert!(result.is_none());

        let result = router.resolve("GET", "/issue/test");
        let (handler, params) = result.unwrap();
        assert_eq!(handler, 3);
        assert_eq!(params, vec![("issue", "test")]);

        let result = router.resolve("GET", "/static/");
        let (handler, params) = result.unwrap();
        assert_eq!(handler, 4);
        assert_eq!(params, vec![]);

        let result = router.resolve("GET", "/static");
        let (handler, params) = result.unwrap();
        assert_eq!(handler, 5);
        assert_eq!(params, vec![]);
    }
}
