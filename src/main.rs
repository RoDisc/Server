#[macro_use] extern crate rocket;

use rocket::serde::{json::Json, Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use rand::Rng;
use tokio::time::{sleep, Duration};

#[derive(Serialize, Deserialize, Clone)]
struct UserCode {
    username: String,
    code: u32,
}

type CodeList = Arc<Mutex<Vec<UserCode>>>;

#[derive(Deserialize)]
struct CodeRequest {
    username: String,
}

#[derive(Deserialize)]
struct CodeValidationRequest {
    code: u32,
}

#[derive(Serialize)]
struct ValidationResponse {
    username: String,
    valid: bool,
    message: String,
}

#[post("/generate", format = "json", data = "<code_request>")]
async fn generate_code(
    code_request: Json<CodeRequest>,
    codes: &rocket::State<CodeList>
) -> Json<UserCode> {
    let username = code_request.username.clone();

    let mut rng = rand::thread_rng();
    let code: u32 = rng.gen_range(100000..1000000);

    let user_code = UserCode { username: username.clone(), code };
    let mut codes_locked = codes.lock().unwrap();
    codes_locked.push(user_code.clone());

    let codes_clone = Arc::clone(&codes);

    tokio::spawn(async move {
        sleep(Duration::from_secs(600)).await;
        let mut codes_locked = codes_clone.lock().unwrap();
        codes_locked.retain(|x| x.code != code); 
    });

    Json(user_code)
}

#[get("/list")]
fn list_codes(codes: &rocket::State<CodeList>) -> Json<Vec<UserCode>> {
    let codes_locked = codes.lock().unwrap();
    Json(codes_locked.clone())
}

#[post("/validate", format = "json", data = "<validation_request>")]
fn validate_code(
    validation_request: Json<CodeValidationRequest>,
    codes: &rocket::State<CodeList>
) -> Json<ValidationResponse> {
    let codes_locked = codes.lock().unwrap();

    if let Some(user_code) = codes_locked.iter().find(|&x| x.code == validation_request.code) {

        Json(ValidationResponse {
            username: user_code.username.clone(),
            valid: true,
            message: format!("Code {} is valid.", validation_request.code),
        })
    } else {
        Json(ValidationResponse {
            username: "".to_string(),
            valid: false,
            message: format!("Code {} is invalid or expired.", validation_request.code),
        })
    }
}

#[launch]
fn rocket() -> _ {
    let codes: CodeList = Arc::new(Mutex::new(Vec::new()));
    rocket::build()
        .manage(codes)
        .mount("/", routes![generate_code, list_codes, validate_code])
}