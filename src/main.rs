#[macro_use] extern crate rocket;
mod avl;
mod handlers;


#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(avl::AVLTreeSingleton::new())
        .mount("/", routes![handlers::set, handlers::get, handlers::delete])
}