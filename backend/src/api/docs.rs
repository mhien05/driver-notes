use super::{auth, backup, entries, shifts};
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

#[derive(OpenApi)]
#[openapi(
    info(title = "Driver Notes API", version = "1.0.0", description = "API ghi ca làm, thu nhập và chi phí cho tài xế. Thời gian dùng Unix milliseconds; tiền dùng số nguyên VND."),
    paths(
        auth::register, auth::login, auth::logout, auth::me,
        shifts::list_shifts, shifts::start_shift, shifts::end_shift,
        shifts::replace_shift, shifts::delete_shift,
        entries::list_entries, entries::create_entry,
        entries::replace_entry, entries::delete_entry,
        backup::snapshot, backup::restore
    ),
    components(schemas(
        auth::Credentials, auth::AuthResponse, auth::Profile,
        shifts::Shift, shifts::ShiftInput,
        entries::Entry, entries::EntryInput,
        backup::Snapshot, backup::Backup, backup::RestoreResult
    )),
    modifiers(&BearerSecurity),
    security(("bearerAuth" = []))
)]
struct ApiDoc;

struct BearerSecurity;

impl Modify for BearerSecurity {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearerAuth",
                SecurityScheme::Http(HttpBuilder::new().scheme(HttpAuthScheme::Bearer).build()),
            );
        }
    }
}

pub(crate) fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}
