mod preview;
pub use preview::*;

mod transform;
pub use transform::*;

mod jobs;
pub use jobs::*;

mod root;
pub use root::*;

use crate::models::JobModel;

pub trait GetSelfRoute: Clone {
    fn get_self_route(&self) -> String;
}

impl<InputType, ResultType> JobModel<InputType, ResultType> where JobModel<InputType, ResultType>: GetSelfRoute, ResultType: Clone {
    pub fn to_dto(&self) -> JobDto<ResultType> {
        JobDto {
            id: self.id.clone(),
            status: self.status.clone(),
            message: self.message.clone(),
            result: self.result.clone(),
            _links: JobLinks {
                _self: self.get_self_route()
            }
        }
    }
}
