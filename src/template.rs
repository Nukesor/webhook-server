use ::std::collections::HashMap;
use ::actix_web::HttpResponse;
use ::actix_web::http::StatusCode;
use ::handlebars::Handlebars;
use ::log::info;


pub fn verify_template_parameters(template: String, params: &HashMap<String, String>) -> Result<(), HttpResponse> {

    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(true);
    info!("Got parameters: {:?}", params);
    let result = handlebars.render_template(&template, params);

    match result {
        Err(error) => {Err(
            HttpResponse::build(StatusCode::BAD_REQUEST).json(format!("{:?}", error))
        )},
        Ok(result) => {
            info!("Template renders properly: {}", result);
            Ok(())
        }
    }
}
