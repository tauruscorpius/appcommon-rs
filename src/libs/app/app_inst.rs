use lazy_static::lazy_static;
use tokio::sync::Mutex;
use strum_macros::{EnumString, Display};

#[derive(Debug, Display, EnumString, Clone, PartialEq)]
pub enum AppStatus {
    #[strum(serialize = "init")]
    INIT,
    #[strum(serialize = "running")]
    RUNNING,
    #[strum(serialize = "exiting")]
    EXITING(String),
    #[strum(serialize = "exited")]
    EXITED
}

impl Default for AppStatus {
    fn default() -> Self {
        Self::INIT
    }
}

#[derive(Debug, Default)]
pub struct AppInstance {
    pub _application_uuid : Mutex<String>,
    pub app_status : Mutex<AppStatus>,
    pub _service_api_root : Mutex<String>,
    pub _app_service_type : Mutex<String>,
}

impl AppInstance {
    pub async fn get_register_info(&self,
                                   http: bool,
                                   ty : String
    ) -> (String, String, String, String) {
        let mut scheme = String::default();
        if http {
            scheme = "http".to_string();
        }
        (scheme,
         self.get_service_api_root().await,
         ty.to_string(),
         self.get_application_uuid().await,
        )
    }
    pub async fn get_application_uuid(&self) -> String {
        Self::get_mutex_value(&self._application_uuid).await
    }
    pub async fn get_app_status(&self) -> AppStatus {
        Self::get_mutex_value(&self.app_status).await
    }
    pub async fn get_service_api_root(&self) -> String {
        Self::get_mutex_value(&self._service_api_root).await
    }
    pub async fn get_app_service_type(&self) -> String {
        Self::get_mutex_value(&self._app_service_type).await
    }

    pub async fn set_application_uuid(&self, v : String) {
        Self::set_mutex_value(&self._application_uuid, v).await
    }
    pub async fn set_app_status(&self, v : AppStatus) {
        Self::set_mutex_value(&self.app_status, v).await
    }
    pub async fn set_service_api_root(&self, v : String) {
        Self::set_mutex_value(&self._service_api_root, v).await
    }
    pub async fn set_app_service_type(&self, v : String) {
        Self::set_mutex_value(&self._app_service_type, v).await
    }

    async fn set_mutex_value<T>(a : &Mutex<T>, value : T)
        where T : Clone
    {
        let mut v = a.lock().await;
        *v = value.clone()
    }
    async fn get_mutex_value<T>(a : &Mutex<T>) -> T
        where T : Clone
    {
        let v = a.lock().await;
        v.clone()
    }
}

lazy_static!(
  static ref SINGLETON_INSTANCE_APP : AppInstance = AppInstance{
        _application_uuid : Mutex::new(uuid::Uuid::new_v4().to_string()),
        app_status : Mutex::new(AppStatus::INIT),
        .. Default::default()
    };
);

pub fn get_app_instance() -> &'static AppInstance {
    &*SINGLETON_INSTANCE_APP
}