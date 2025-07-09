#[derive(Clone, Debug)]
pub struct Env {
    pub redis_host: String,
    pub redis_port: u16,
    pub redis_username: String,
    pub redis_password: String,
}

impl Default for Env {
    fn default() -> Env {
        Env { 
            redis_host: "127.0.0.1".to_owned(), 
            redis_port: 6379,
            redis_password: "".to_owned(),
            redis_username: "".to_owned(),  
        }
    }
}

impl Env {
    pub fn from_env() -> Env {
        let mut env = Env::default();
        if let Ok(host) = std::env::var("REDIS_HOST") {
            env.redis_host = host;
        }
        if let Ok(port) = std::env::var("REDIS_PORT") {
            env.redis_port = port.parse::<u16>().expect("u16 REDIS_PORT");
        }
        if let Ok(username) = std::env::var("REDIS_USERNAME") {
            env.redis_username = username;
        }
        if let Ok(password) = std::env::var("REDIS_PASSWORD") {
            env.redis_password = password;
        }
        // Also check for REDIS_AUTH_PASSWORD as fallback
        if env.redis_password.is_empty() {
            if let Ok(auth_password) = std::env::var("REDIS_AUTH_PASSWORD") {
                env.redis_password = auth_password;
            }
        }

        env
    }

    pub fn to_redis_uri(&self) -> String {
        let host = &self.redis_host;
        let port = self.redis_port;
        let username = &self.redis_username;
        let password = &self.redis_password;
        
        // Build Redis URI based on whether we have username/password
        if !password.is_empty() {
            if !username.is_empty() {
                format!("redis://{}:{}@{}:{}", username, password, host, port)
            } else {
                // No username, just password (standard Redis auth)
                format!("redis://:{}@{}:{}", password, host, port)
            }
        } else {
            // No authentication
            format!("redis://{}:{}", host, port)
        }
    }
}
