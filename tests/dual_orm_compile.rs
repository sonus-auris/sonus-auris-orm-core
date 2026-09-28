#[test]
fn diesel_and_seaorm_are_buildable() {
    let diesel_backend = std::any::type_name::<diesel::pg::Pg>();
    let seaorm_error = std::any::type_name::<sea_orm::DbErr>();

    assert!(diesel_backend.contains("Pg"));
    assert!(seaorm_error.contains("DbErr"));
}
