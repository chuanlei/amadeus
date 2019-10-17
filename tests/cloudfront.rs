#![allow(where_clauses_object_safety)]

use serde_closure::FnMut;

use amadeus::prelude::*;
#[cfg(feature = "constellation")]
use constellation::*;
use std::{
	env, time::{Duration, SystemTime}
};

fn main() {
	#[cfg(feature = "constellation")]
	init(Resources::default());

	// Accept the number of processes at the command line, defaulting to 10
	let processes = env::args()
		.nth(1)
		.and_then(|arg| arg.parse::<usize>().ok())
		.unwrap_or(10);

	let local_pool_time = {
		// let local_pool = LocalPool::new();
		0 // run(&local_pool) // cloudfront too slow
	};
	let thread_pool_time = {
		let thread_pool = ThreadPool::new(processes).unwrap();
		run(&thread_pool)
	};
	#[cfg(feature = "constellation")]
	let process_pool_time = {
		let process_pool = ProcessPool::new(processes, 1, Resources::default()).unwrap();
		run(&process_pool)
	};
	#[cfg(not(feature = "constellation"))]
	let process_pool_time = "-";

	println!(
		"in {:?} {:?} {:?}",
		local_pool_time, thread_pool_time, process_pool_time
	);
}

fn run<P: amadeus_core::pool::ProcessPool>(pool: &P) -> Duration {
	let start = SystemTime::now();

	let _ = DistributedIteratorMulti::<&Result<CloudfrontRow, AwsError>>::count(Identity);

	let ((), (count, count2)) = Cloudfront::new(
		AwsRegion::UsEast1,
		"us-east-1.data-analytics",
		"cflogworkshop/raw/cf-accesslogs/",
	)
	.unwrap()
	.dist_iter()
	.multi(
		pool,
		Identity.for_each(FnMut!(|x: Result<CloudfrontRow, _>| {
			let _x = x.unwrap();
			// println!("{:?}", x.url);
		})),
		(
			Identity.map(FnMut!(|_x: &Result<_, _>| {})).count(),
			Identity.cloned().count(),
			// DistributedIteratorMulti::<&Result<CloudfrontRow, AwsError>>::count(Identity),
		),
	);
	assert_eq!(count, count2);
	assert_eq!(count, 207_928);

	start.elapsed().unwrap()
}