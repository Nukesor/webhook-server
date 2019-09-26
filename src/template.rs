use ::actix_web::http::StatusCode;
use ::actix_web::HttpResponse;
use ::handlebars::Handlebars;
use ::log::info;
use ::std::collections::HashMap;

pub fn verify_template_parameters(
    template: String,
    params: &HashMap<String, String>,
) -> Result<(), HttpResponse> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(true);
    info!("Got parameters: {:?}", params);
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
