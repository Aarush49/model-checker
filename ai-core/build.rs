fn main() {
    println!("cargo:rerun-if-changed=.env");

    if let Ok(env_iter) = dotenvy::dotenv_iter() {
        for item_result in env_iter {
            if let Ok((key, value)) = item_result {
                println!("cargo:rustc-env={}={}", key, value);
            }
        }
    }
}
