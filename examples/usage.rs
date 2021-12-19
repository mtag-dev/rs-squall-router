use squall_router::SquallRouter;

fn main() {
    let mut router = SquallRouter::new();

    router.add_validator("int".to_string(), r"[0-9]+".to_string());
    router.add_validator(
        "uuid".to_string(),
        r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}".to_string(),
    );

    router.add_http_route(
        "GET".to_string(),
        "/route/without/dynamic/octets".to_string(),
        0,
    );
    router.add_http_route(
        "GET".to_string(),
        "/route/aaa/{string_param}/bbb/{num_param:int}/ccc/{uuid_param:uuid}".to_string(),
        1,
    );
    router.add_http_location("GET".to_string(), "/files/css".to_string(), 2);

    let (handler_0, names_0, values_0) = router
        .get_http_handler("GET", "/route/without/dynamic/octets")
        .unwrap();
    assert_eq!(handler_0, 0);

    let (handler_1, names_1, values_1) = router
        .get_http_handler(
            "GET",
            "/route/aaa/aaa_value/bbb/1234/ccc/4bea5a51-1b80-4433-be06-d52726015591",
        )
        .unwrap();
    assert_eq!(handler_1, 1);
    assert_eq!(names_1, vec!["string_param", "num_param", "uuid_param"]);
    assert_eq!(
        values_1,
        vec!["aaa_value", "1234", "4bea5a51-1b80-4433-be06-d52726015591"]
    );

    let (handler_2, names_2, values_2) = router
        .get_http_handler("GET", "/files/css/vendor/style.css")
        .unwrap();
    assert_eq!(handler_2, 2);
}
