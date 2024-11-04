use futures_lite::future;
use saras::default_app;
use clap::{Parser, Subcommand};
use saras::auth::hashing;
//use lpsql::db::get_pool;
//use std::sync::Arc;
//use lpsql::pool::ConnectionPool;
//use lpsql::QueryParam as qp;
//use async_std::task;



#[derive(Subcommand, Debug)]
enum Commands {
    HashPwd { pwd: String },
}


#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    commands: Option<Commands>,
}

//async fn fetch_users(pool: Arc<ConnectionPool>) {
//	let conn = pool.get_conn().await;
//	let prms: Vec<qp> = vec![
//		//qp::Number(1),
//	];
//	//let q = "select id from users_users where id > $1::INT;";
//	let q = "select pg_sleep(5)";
//	let r = conn.exec(q, prms).await;
//	pool.release_conn(conn).await;
//	println!("done: {r:?}");
//}

//async fn amain() {
//	let pool = get_pool().clone();
//	let mut tasks = vec![];
//
//    for _ in 0..22 {
//        let pool_clone = pool.clone();
//        let task = task::spawn(async move {
//            fetch_users(pool_clone).await;
//        });
//        tasks.push(task);
//    }
//
//    for task in tasks {
//        task.await;
//    }
//}


fn main() {
	let cli = Cli::parse();

    match &cli.commands {
        Some(Commands::HashPwd { pwd}) => {
			let hash = hashing::hash_pwd(pwd);
			println!("hash: {hash}");
        }
        None => {
            println!("Running default app.");
			//future::block_on(amain());
			future::block_on(default_app::run());
        }
    }
}
