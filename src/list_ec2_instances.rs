extern crate rusoto_core;
extern crate rusoto_ec2;

use std::default::Default;
use std::fmt;

use rusoto_core::Region;
use rusoto_ec2::{Filter, Ec2, Ec2Client, DescribeInstancesRequest, Instance, Reservation};

struct InstanceSummary {
    instance: Instance,
}

impl fmt::Display for InstanceSummary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let public_ip_address = self.find_public_ip_address().unwrap_or("-".to_string());
        let name = self.find_name_in_tags().unwrap_or("-".to_string());
        write!(f, "{}\t{}", name, public_ip_address)
    }
}

impl InstanceSummary {
    fn find_name_in_tags(&self) -> Option<String> {
        self.instance.tags.as_ref()
            .and_then(|ts| ts.iter().find(|&tag| tag.key == Some("Name".to_string())))
            .and_then(|tag| tag.value.as_ref().and_then(|ref v| Some(v.to_string())))
    }

    fn find_public_ip_address(&self) -> Option<String> {
        self.instance.public_ip_address.as_ref().map(|s| s.to_string())
    }
}


fn show_instances(is: &Vec<Instance>) {
    for i in is {
        let summary = InstanceSummary { instance: i.to_owned() };
        println!("{}", summary);
    }
}

fn show_reservations(rs: &Vec<Reservation>) {
    for r in rs {
        r.instances.as_ref().map(|is| show_instances(&is));
    }
}

fn create_request() -> DescribeInstancesRequest {
    let filter = Filter {
        name: Some("instance-state-name".to_string()),
        values: Some(vec!["running".to_string()])
    };

    DescribeInstancesRequest {
        filters: Some(vec![filter]),
        ..Default::default()
    }
}

fn main() {
  let client = Ec2Client::new(Region::ApNortheast1);
  let request = create_request();

  match client.describe_instances(request).sync() {
    Ok(output) => {
      match output.reservations {
        Some(reservations) => {
            show_reservations(&reservations)
        }
        None => eprintln!("No reservation"),
      }
    }
    Err(error) => {
      eprintln!("Error: {:?}", error);
    }
  }
}
