extern crate rusoto;
use rusoto::{ProfileProvider, Region};
use rusoto::ec2::{Ec2Client, DescribeInstancesRequest};

extern crate rustc_serialize;
extern crate docopt;
use docopt::Docopt;

const USAGE: &'static str = "
Usage:
    list-ec2-instances
    list-ec2-instances --profile=<profile>
    list-ec2-instances --help

Options:
    --profile=<profile>  Use a specific profile from your credential file.
";

#[derive(Debug, RustcDecodable)]
pub struct Args {
    flag_profile: Option<String>,
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

fn main() {
    let args: Args = parse_args();

    let provider = find_provider(&args).unwrap();
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
