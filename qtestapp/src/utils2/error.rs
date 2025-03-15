use warp::reject::Reject;
// Define errores personalizados
#[derive(Debug)]
pub struct InvalidPeripheralName;
impl Reject for InvalidPeripheralName {}

#[derive(Debug)]
pub struct CustomError;
impl Reject for CustomError {}