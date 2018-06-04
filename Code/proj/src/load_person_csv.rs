use csv;

use std::error::Error;
use std::io;
use std::process;
use std::env;
use std::ffi::OsString;
use std::fs::File;

// Nodes @ social_network/person_0_0.csv
// Edges @ social_network/person_knows_person_0_0.csv

pub fn social_network_sf1() -> &'static str {
    return "/home/rob/S3G2/ldbc_snb_datagen/social_network_sf1"
}

pub fn social_network_sf10() -> &'static str {
    return "/home/rob/S3G2/ldbc_snb_datagen/social_network_sf10"
}

pub fn nodes(social_network: &str) -> String {
    return social_network.to_string() + "/person_0_0.csv"
}

pub fn edges(social_network: &str) -> String {
    return social_network.to_string() + "/person_knows_person_0_0.csv"
}

// NODES/PEOPLE

#[derive(Deserialize)]
struct RawPerson {
    id: i64,
    firstName: String,
    lastName: String,
    gender: String,
    birthday: String,
    creationDate: String,
    locationIP: String,
    browserUsed: String,
}

#[derive(Debug)]
pub struct Person {
    id: i64,
    first_name: String,
    last_name: String,
}

fn read_people(file_path: String) -> Result<Vec<Person>, Box<Error>> {
    let file = File::open(file_path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'|')
        .from_reader(file);
    let mut people: Vec<Person> = Vec::new();
    for result in rdr.deserialize() {
        // Notice that we need to provide a type hint for automatic
        // deserialization.
        let raw_person: RawPerson = result?;
        let person = Person { id: raw_person.id, first_name: raw_person.firstName, last_name: raw_person.lastName};
        people.push(person);
    }
    Ok(people)
}

pub fn read_nodes(file_path: String) -> Vec<Person> {
    match read_people(file_path) {
        Ok(people) => people,
        Err(err)   => panic!(),
    }
}

// EDGES/FRIENDSHIPS

#[derive(Debug,Deserialize)]
struct RawFriendship {
    p1id: i64,
    p2id: i64,
    creationDate: String,
}

pub struct Friendship {
    p1id: i64,
    p2id: i64,
}

fn read_friendships(file_path: String) -> Result<Vec<Friendship>, Box<Error>> {
    let file = File::open(file_path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'|')
        .comment(Some(b'P')) // Ignore the header line because it has Person.id|Person.id which confuses this library.
        .from_reader(file);
    let mut friendships: Vec<Friendship> = Vec::new();
    for result in rdr.deserialize() {
        // Notice that we need to provide a type hint for automatic
        // deserialization.
        let raw_friendship: RawFriendship = result?;
        let friendship = Friendship { p1id: raw_friendship.p1id, p2id: raw_friendship.p2id };
        friendships.push(friendship);
    }
    Ok(friendships)
}

fn read_friendships_adjl(file_path: String) -> Result<(Vec<i64>, Vec<i64>), Box<Error>> {
    let file = File::open(file_path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'|')
        .comment(Some(b'P')) // Ignore the header line because it has Person.id|Person.id which confuses this library.
        .from_reader(file);
    let mut src: Vec<i64> = Vec::new();
    let mut dst: Vec<i64> = Vec::new();
    for result in rdr.deserialize() {
        // Notice that we need to provide a type hint for automatic
        // deserialization.
        let raw_friendship: RawFriendship = result?;
        src.push(raw_friendship.p1id);
        dst.push(raw_friendship.p2id);
    }
    Ok((src, dst))
}

pub fn read_edges(file_path: String) -> Vec<Friendship> {
    match read_friendships(file_path) {
        Ok(friendships) => friendships,
        Err(err)        => panic!(),
    }
}

pub fn read_edges_adjl(file_path: String) -> (Vec<i64>, Vec<i64>) {
    match read_friendships_adjl(file_path) {
        Ok((src, dst)) => (src, dst),
        Err(err)        => panic!(),
    }
}

// Data Access

pub fn small_nodes() -> Vec<Person> {
    read_nodes(nodes(social_network_sf1()))
}

pub fn small_edges() -> Vec<Friendship> {
    read_edges(edges(social_network_sf1()))
}

pub fn small_edges_adjl() -> (Vec<i64>, Vec<i64>) {
    read_edges_adjl(edges(social_network_sf1()))
}

pub fn big_nodes() -> Vec<Person> {
    read_nodes(nodes(social_network_sf10()))
}

pub fn big_edges() -> Vec<Friendship> {
    read_edges(edges(social_network_sf10()))
}

pub fn big_edges_adjl() -> (Vec<i64>, Vec<i64>) {
    read_edges_adjl(edges(social_network_sf10()))
}

pub fn test() {
    let people = small_nodes();
    for i in 0..10 {
        println!("{:?}", people[i]);
    }
    println!("...");
    println!("Successfully read {} people", people.len());

    let (src, dst) = small_edges_adjl();
    for i in 0..10 {
        println!("{} -> {}", src[i], dst[i]);
    }
    println!("...");
    println!("Successfully read {} friendships", src.len());
}