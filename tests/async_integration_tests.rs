use actix_web::{test, web, App, http::StatusCode};

#[actix_web::test]
async fn test_health_endpoint() {
    let app = test::init_service(
        App::new()
            .route("/health", web::get().to(|| async { "OK" }))
    ).await;
    
    let req = test::TestRequest::get()
        .uri("/health")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_404_for_unknown_routes() {
    let app = test::init_service(
        App::new()
            .route("/api/test", web::get().to(|| async { "test" }))
    ).await;
    
    let req = test::TestRequest::get()
        .uri("/api/unknown")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}