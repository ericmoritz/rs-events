// This serves as the public API for the events service
extern crate chrono;
extern crate failure;


use chrono::{DateTime, Utc};
use failure::Error;
use super::user;

pub trait EventService {
    
    // event gets an event if it exists
    fn event(request: EventRequest) -> Result<Option<Event>, Error>;

    // create_event
    fn create_event(request: CreateEventRequest) -> Result<Event, Error>;

    // rsvps gets the RSVPs for an event
    fn event_rsvps(request: EventRsvpRequest) -> Result<Collection<RsvpAction>, Error>;

    // rsvp for event
    fn rsvp_for_event(request: RsvpForEventRequest) -> Result<RsvpAction, Error>;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventRequest {
    // https://schema.org/Thing properties
    identifier: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateEventRequest {
    // The access_token retrieved using the user service
    access_token: String, 

    // https://schema.org/Thing properties
    description: String,
    #[serde(rename = "disambiguatingDescription")]
    disambiguating_description: String, // The event's "tag line"
    identifier: String,
    name: String,
    url: String,

    // https://schema.org/Event properties
    #[serde(rename = "endDate")]
    end_date: DateTime<Utc>,
    location: Place,
    organizer: Person,
    #[serde(rename = "startDate")]
    start_date: DateTime<Utc>, 
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventRsvpRequest {
    target_identifier: String
}

// Event is a https://schema.org/Event
#[derive(Serialize, Deserialize, Debug)]
pub struct Event {

    // https://schema.org/Thing properties
    description: String,
    #[serde(rename = "disambiguatingDescription")]
    disambiguating_description: String, // The event's "tag line"
    identifier: String,
    name: String,
    url: String,

    // https://schema.org/Event properties
    #[serde(rename = "endDate")]
    end_date: DateTime<Utc>,
    location: Place,
    organizer: Person,
    #[serde(rename = "startDate")]
    start_date: DateTime<Utc>,
}

// Place is a https://schema.org/Place
#[derive(Serialize, Deserialize, Debug)]
pub struct Place {
    // https://schema.org/Thing properties
    name: String,

    // https://schema.org/Place properties
    address: String,

}

// Person is a https://schema.org/Person
#[derive(Serialize, Deserialize, Debug)]
pub struct Person {
    // https://schema.org/Thing properties
    identifier: String,
    name: String
}

// RsvpAction is a https://schema.org/RsvpAction
#[derive(Serialize, Deserialize, Debug)]
pub struct RsvpAction {
    // https://schema.org/RsvpAction properties
    agent: Option<Person>, // None means the Action is anonymous
    #[serde(rename = "rsvpResponse")]
    rsvp_response: String, // {yes,no,maybe}
    target: Event,
}

// http://www.w3.org/ns/hydra/core#Collection
#[derive(Serialize, Deserialize, Debug)]
pub struct Collection<T> {
    member: Vec<T>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RsvpForEventRequest {
    // The access_token retrieved using the user service
    access_token: user::AccessToken,    
}
