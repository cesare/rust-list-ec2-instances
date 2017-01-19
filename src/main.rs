extern crate rusoto;
use rusoto::{DefaultCredentialsProvider, Region};
use rusoto::ec2::{Ec2Client, DescribeHostsRequest};

fn main() {
    let provider = DefaultCredentialsProvider::new().unwrap();
    let region = Region::ApNortheast1;
    let client = Ec2Client::new(provider, region);
    let request = DescribeHostsRequest::default();

    match client.describe_hosts(&request) {
        Ok(results) => {
            results.hosts.map(|hosts| for host in hosts {
                println!("{:?}", host);
            });
        }
        Err(error) => println!("{:?}", error),
    }
}
