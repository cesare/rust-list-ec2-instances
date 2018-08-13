extern crate rusoto_core;
extern crate rusoto_credential;
extern crate rusoto_ec2;

use rusoto_credential::ProfileProvider;
use rusoto_core::{Region, CredentialsError};
use rusoto_core::region::ParseRegionError;
use rusoto_core::request::HttpClient;
use rusoto_ec2::{DescribeInstancesRequest, DescribeInstancesResult, Ec2, Ec2Client, Instance,
                 Reservation};

extern crate docopt;
extern crate rustc_serialize;
use docopt::Docopt;

extern crate futures;
use futures::future::Future;

use std::str::FromStr;

struct InstanceStatus {
    id: Option<String>,
    ip_address: Option<String>,
    name: Option<String>,
}

impl InstanceStatus {
    fn from_instance(instance: &Instance) -> InstanceStatus {
        InstanceStatus {
            id: instance.instance_id.clone(),
            ip_address: instance.public_ip_address.clone(),
            name: Self::find_instance_name(instance),
        }
    }

    fn find_instance_name(instance: &Instance) -> Option<String> {
        instance.tags.as_ref().and_then(|tags|
            tags.iter()
                .find(|tag| tag.key.as_ref().map_or(false, |key| key == "Name"))
                .and_then(|tag| tag.value.clone())
        )
    }
}

impl std::fmt::Display for InstanceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, " {:<10}  {:^15}  {:<}",
            self.id.as_ref().unwrap_or(&"(unknown)".to_owned()),
            self.ip_address.as_ref().unwrap_or(&"-".to_owned()),
            self.name.as_ref().unwrap_or(&"-".to_owned())
        )
    }
}

const USAGE: &'static str = "
Usage:
    list-ec2-instances
    list-ec2-instances [--profile=<profile>] [--region=<region>]
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
    Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit())
}

fn find_provider(args: &Args) -> Result<ProfileProvider, CredentialsError> {
    let mut provider = ProfileProvider::new()?;
    args.flag_profile.as_ref()
        .map(|profile| provider.set_profile(profile.to_owned()));
    Ok(provider)
}

fn find_region(args: &Args) -> Result<Region, ParseRegionError> {
    args.flag_region.as_ref()
        .map_or(Ok(Region::ApNortheast1), |value| Region::from_str(&value))
}

fn show_instances(is: &Vec<Instance>) {
    for i in is {
        println!("{}", InstanceStatus::from_instance(i));
    }
}

fn show_reservations(rs: &Vec<Reservation>) {
    for r in rs {
        r.instances.as_ref().map(|is| show_instances(&is));
    }
}

fn show_result(result: DescribeInstancesResult) {
    result.reservations.map(|rs| show_reservations(&rs));
}

fn main() {
    let args: Args = parse_args();

    let provider = find_provider(&args).unwrap();
    let region = find_region(&args).unwrap();
    let client = Ec2Client::new_with(HttpClient::new().unwrap(), provider, region);
    let request = DescribeInstancesRequest::default();

    let _ = client
        .describe_instances(request)
        .map(|result| show_result(result))
        .map_err(|e| eprintln!("Error: {}", e))
        .wait();
}
