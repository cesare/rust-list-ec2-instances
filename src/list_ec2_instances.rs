extern crate rusoto_core;
extern crate rusoto_ec2;

use std::default::Default;

use rusoto_core::Region;
use rusoto_ec2::{Filter, Ec2, Ec2Client, DescribeInstancesRequest, Instance, Reservation, Tag};

fn find_name_in_tags(tags: &Option<Vec<Tag>>) -> Option<String> {
    tags.as_ref()
        .and_then(|ts| ts.iter().find(|&tag| tag.key == Some("Name".to_string())))
        .and_then(|tag| tag.value.as_ref().and_then(|ref v| Some(v.to_string())))
}

fn show_instances(is: &Vec<Instance>) {
    for i in is {
        let default = "-".to_string();
        let public_ip_address = i.public_ip_address.as_ref().unwrap_or(&default);
        let name = find_name_in_tags(&i.tags).unwrap_or("-".to_string());
        println!("{}\t{}", name, public_ip_address);
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

    let request = DescribeInstancesRequest {
        filters: Some(vec![filter]),
        ..Default::default()
    };
    request
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
        None => println!("No reservation"),
      }
    }
    Err(error) => {
      println!("Error: {:?}", error);
    }
  }
}
