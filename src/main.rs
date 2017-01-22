extern crate rusoto;
use rusoto::{ProfileProvider, Region};
use rusoto::ec2::{Ec2Client, DescribeInstancesRequest, Instance};

extern crate rustc_serialize;
extern crate docopt;
use docopt::Docopt;

use std::error::Error;
use std::str::FromStr;

const USAGE: &'static str = "
Usage:
    list-ec2-instances
    list-ec2-instances --profile=<profile> --region=<region>
    list-ec2-instances --help

Options:
    --profile=<profile>  Use a specific profile from your credential file.
    --region=<region>    Specify a region.
";

#[derive(Debug, RustcDecodable)]
pub struct Args {
    flag_profile: Option<String>,
    flag_region: Option<String>,
}

fn parse_args() -> Args {
    Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit())
}

fn find_provider(args: &Args) -> Result<ProfileProvider, String> {
    match ProfileProvider::new() {
        Ok(mut provider) => {
            if let Some(ref name) = args.flag_profile {
                provider.set_profile(name.to_string());
            }
            Ok(provider)
        },
        Err(_) => Err(String::from("Failed to find provider"))
    }
}

fn find_region(args: &Args) -> Result<Region, String> {
    match args.flag_region {
        Some(ref value) => {
            Region::from_str(value).or_else(|e| Err(String::from(e.description())))
        }
        None => Ok(Region::ApNortheast1),
    }
}

fn show_instance(i: &Instance) {
    println!("{:?} {:?} {:?}", i.instance_id, i.state, i.public_ip_address);
}

fn main() {
    let args: Args = parse_args();

    let provider = find_provider(&args).unwrap();
    let region = find_region(&args).unwrap();
    let client = Ec2Client::new(provider, region);
    let request = DescribeInstancesRequest::default();

    match client.describe_instances(&request) {
        Ok(results) => {
            results.reservations.map(|rs| for r in rs {
                r.instances.map(|is| for i in is {
                    show_instance(&i);
                });
            });
        }
        Err(error) => println!("{:?}", error),
    }
}
