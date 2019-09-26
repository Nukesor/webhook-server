use ::actix_web::http::StatusCode;
use ::actix_web::HttpResponse;
use ::handlebars::Handlebars;
use ::log::info;
use ::std::collections::HashMap;


/// Verify that the template renders with the given parameters
pub fn verify_template_parameters(
    template: String,
    params: &HashMap<String, String>,
) -> Result<(), HttpResponse> {
    info!("Got parameters: {:?}", params);
    // Create a new handlebar instance and enable strict mode to prevent missing or malformed arguments
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(true);

    // Check the template for render errors with the current parameter
    let result = handlebars.render_template(&template, params);
    match result {
        Err(error) => {
            Err(HttpResponse::build(StatusCode::BAD_REQUEST).json(format!("{:?}", error)))
        }
        Ok(result) => {
            info!("Template renders properly: {}", result);
            Ok(())
        }
    }
}
