use squall_router::SquallRouter;

fn main() {
    let mut router = SquallRouter::new();

    router
        .add_validator("int".to_string(), r"[0-9]+".to_string())
        .unwrap();
    router
        .add_validator(
            "uuid".to_string(),
            r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}".to_string(),
        )
        .unwrap();

    router
        .add_route(
            "GET".to_string(),
            "/route/without/dynamic/octets".to_string(),
            0,
        )
        .unwrap();
    router
        .add_route(
            "GET".to_string(),
            "/route/aaa/{string_param}/bbb/{num_param:int}/ccc/{uuid_param:uuid}".to_string(),
            1,
        )
        .unwrap();
    router.add_location("GET".to_string(), "/files/css".to_string(), 2);

    let (handler_0, _parameters_0) = router
        .resolve("GET", "/route/without/dynamic/octets")
        .unwrap();
    assert_eq!(handler_0, 0);

    let (handler_1, parameters_1) = router
        .resolve(
            "GET",
            "/route/aaa/aaa_value/bbb/1234/ccc/4bea5a51-1b80-4433-be06-d52726015591",
        )
        .unwrap();
    assert_eq!(handler_1, 1);
    assert_eq!(
        parameters_1,
        vec![
            ("string_param", "aaa_value"),
            ("num_param", "1234"),
            ("uuid_param", "4bea5a51-1b80-4433-be06-d52726015591")
        ]
    );

    let (handler_2, _parameters_2) = router
        .resolve("GET", "/files/css/vendor/style.css")
        .unwrap();
    assert_eq!(handler_2, 2);
}
