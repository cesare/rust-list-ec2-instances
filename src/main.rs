extern crate rusoto;
use rusoto::{ProfileProvider, Region};
use rusoto::ec2::{Ec2Client, DescribeInstancesRequest};

fn main() {
    let provider = ProfileProvider::new().unwrap();
    let region = Region::ApNortheast1;
    let client = Ec2Client::new(provider, region);
    let request = DescribeInstancesRequest::default();

    match client.describe_instances(&request) {
        Ok(results) => {
            results.reservations.map(|rs| for r in rs {
                println!("{:?}", r);
            });
        }
        Err(error) => println!("{:?}", error),
    }
}
