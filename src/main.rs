mod delete;
mod info;
mod logs;
mod policy;
mod spawner;
use std::{
    env,
    //    os
};
use anyhow;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Not Enought Argument !!! \n arg = {:?}", args);
    }
    match args[1].as_ref() {
        "test" => println!("matching Test concluant"),
        "get_pod" => info::pods()?,
        "get_namespaces" => info::namespace()?,
        "get_logs" => logs::get_logs(args[2].clone())?,
        "spawn_namespace" => spawner::spawn_namespace(args[2].clone())?,
        "spawn_pod" => spawner::spawn_pod(args[2].clone())?,
        "del_namespace" => delete::del_namespace(args[2].clone())?,
        "del_pod" => delete::del_pod(args[2].clone())?,
        "deploy" => spawner::deployment(args[2].clone(), args[3].clone())?,
        "exec" => spawner::exec_cmd_pod(args[2].clone())?,
        //"policy" => policy::main ()?;
        //"describe" => describe_node::describe()?,
        _ => println!("None Matched."),
    };
    Ok(())
}
