use std::collections::HashMap;
use std::cell::RefCell;
use std::default::Default;
use std::collections::BTreeMap;
use serde_json as json;
use std::io;
use std::fs;
use std::mem;
use std::thread::sleep;

use crate::client;

// ##############
// UTILITIES ###
// ############

/// Identifies the an OAuth2 authorization scope.
/// A scope is needed when requesting an
/// [authorization token](https://developers.google.com/youtube/v3/guides/authentication).
#[derive(PartialEq, Eq, Hash)]
pub enum Scope {
    /// View your circles and the people and pages in them
    PluCircleRead,

    /// View your basic profile info, including your age range and language
    PluLogin,

    /// Associate you with your personal info on Google
    PluMe,

    /// Send your photos and videos to Google+
    PluMediaUpload,

    /// View your own Google+ profile and profiles visible to you
    PluProfileRead,

    /// View your Google+ posts, comments, and stream
    PluStreamRead,

    /// View your email address
    UserinfoEmail,

    /// See your personal info, including any personal info you've made publicly available
    UserinfoProfile,
}

impl AsRef<str> for Scope {
    fn as_ref(&self) -> &str {
        match *self {
            Scope::PluCircleRead => "https://www.googleapis.com/auth/plus.circles.read",
            Scope::PluLogin => "https://www.googleapis.com/auth/plus.login",
            Scope::PluMe => "https://www.googleapis.com/auth/plus.me",
            Scope::PluMediaUpload => "https://www.googleapis.com/auth/plus.media.upload",
            Scope::PluProfileRead => "https://www.googleapis.com/auth/plus.profiles.read",
            Scope::PluStreamRead => "https://www.googleapis.com/auth/plus.stream.read",
            Scope::UserinfoEmail => "https://www.googleapis.com/auth/userinfo.email",
            Scope::UserinfoProfile => "https://www.googleapis.com/auth/userinfo.profile",
        }
    }
}

impl Default for Scope {
    fn default() -> Scope {
        Scope::PluMe
    }
}



// ########
// HUB ###
// ######

/// Central instance to access all PlusDomains related resource activities
///
/// # Examples
///
/// Instantiate a new hub
///
/// ```test_harness,no_run
/// extern crate hyper;
/// extern crate hyper_rustls;
/// extern crate yup_oauth2 as oauth2;
/// extern crate google_plusdomains1 as plusdomains1;
/// use plusdomains1::{Result, Error};
/// # async fn dox() {
/// use std::default::Default;
/// use oauth2;
/// use plusdomains1::PlusDomains;
/// 
/// // Get an ApplicationSecret instance by some means. It contains the `client_id` and 
/// // `client_secret`, among other things.
/// let secret: oauth2::ApplicationSecret = Default::default();
/// // Instantiate the authenticator. It will choose a suitable authentication flow for you, 
/// // unless you replace  `None` with the desired Flow.
/// // Provide your own `AuthenticatorDelegate` to adjust the way it operates and get feedback about 
/// // what's going on. You probably want to bring in your own `TokenStorage` to persist tokens and
/// // retrieve them from storage.
/// let auth = yup_oauth2::InstalledFlowAuthenticator::builder(
///         secret,
///         yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
///     ).build().await.unwrap();
/// let mut hub = PlusDomains::new(hyper::Client::builder().build(hyper_rustls::HttpsConnector::with_native_roots()), auth);
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.comments().list("activityId")
///              .sort_order("takimata")
///              .page_token("amet.")
///              .max_results(81)
///              .doit().await;
/// 
/// match result {
///     Err(e) => match e {
///         // The Error enum provides details about what exactly happened.
///         // You can also just use its `Debug`, `Display` or `Error` traits
///          Error::HttpError(_)
///         |Error::Io(_)
///         |Error::MissingAPIKey
///         |Error::MissingToken(_)
///         |Error::Cancelled
///         |Error::UploadSizeLimitExceeded(_, _)
///         |Error::Failure(_)
///         |Error::BadRequest(_)
///         |Error::FieldClash(_)
///         |Error::JsonDecodeError(_, _) => println!("{}", e),
///     },
///     Ok(res) => println!("Success: {:?}", res),
/// }
/// # }
/// ```
pub struct PlusDomains<> {
    client: hyper::Client<hyper_rustls::HttpsConnector<hyper::client::connect::HttpConnector>, hyper::body::Body>,
    auth: oauth2::authenticator::Authenticator<hyper_rustls::HttpsConnector<hyper::client::connect::HttpConnector>>,
    _user_agent: String,
    _base_url: String,
    _root_url: String,
}

impl<'a, > client::Hub for PlusDomains<> {}

impl<'a, > PlusDomains<> {

    pub fn new(client: hyper::Client<hyper_rustls::HttpsConnector<hyper::client::connect::HttpConnector>, hyper::body::Body>, authenticator: oauth2::authenticator::Authenticator<hyper_rustls::HttpsConnector<hyper::client::connect::HttpConnector>>) -> PlusDomains<> {
        PlusDomains {
            client,
            auth: authenticator,
            _user_agent: "google-api-rust-client/2.0.4".to_string(),
            _base_url: "https://www.googleapis.com/plusDomains/v1/".to_string(),
            _root_url: "https://www.googleapis.com/".to_string(),
        }
    }

    pub fn activities(&'a self) -> ActivityMethods<'a> {
        ActivityMethods { hub: &self }
    }
    pub fn audiences(&'a self) -> AudienceMethods<'a> {
        AudienceMethods { hub: &self }
    }
    pub fn circles(&'a self) -> CircleMethods<'a> {
        CircleMethods { hub: &self }
    }
    pub fn comments(&'a self) -> CommentMethods<'a> {
        CommentMethods { hub: &self }
    }
    pub fn media(&'a self) -> MediaMethods<'a> {
        MediaMethods { hub: &self }
    }
    pub fn people(&'a self) -> PeopleMethods<'a> {
        PeopleMethods { hub: &self }
    }

    /// Set the user-agent header field to use in all requests to the server.
    /// It defaults to `google-api-rust-client/2.0.4`.
    ///
    /// Returns the previously set user-agent.
    pub fn user_agent(&mut self, agent_name: String) -> String {
        mem::replace(&mut self._user_agent, agent_name)
    }

    /// Set the base url to use in all requests to the server.
    /// It defaults to `https://www.googleapis.com/plusDomains/v1/`.
    ///
    /// Returns the previously set base url.
    pub fn base_url(&mut self, new_base_url: String) -> String {
        mem::replace(&mut self._base_url, new_base_url)
    }

    /// Set the root url to use in all requests to the server.
    /// It defaults to `https://www.googleapis.com/`.
    ///
    /// Returns the previously set root url.
    pub fn root_url(&mut self, new_root_url: String) -> String {
        mem::replace(&mut self._root_url, new_root_url)
    }
}


// ############
// SCHEMAS ###
// ##########
/// There is no detailed description.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Acl {
    /// Description of the access granted, suitable for display.
    pub description: Option<String>,
    /// Whether access is restricted to the domain.
    #[serde(rename="domainRestricted")]
    pub domain_restricted: Option<bool>,
    /// The list of access entries.
    pub items: Option<Vec<PlusDomainsAclentryResource>>,
    /// Identifies this resource as a collection of access controls. Value: "plus#acl".
    pub kind: Option<String>,
}

impl client::Part for Acl {}


/// There is no detailed description.
/// 
/// # Activities
/// 
/// This type is used in activities, which are methods you may call on this type or where this type is involved in. 
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
/// 
/// * [get activities](ActivityGetCall) (response)
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Activity {
    /// Identifies who has access to see this activity.
    pub access: Option<Acl>,
    /// The person who performed this activity.
    pub actor: Option<ActivityActor>,
    /// Street address where this activity occurred.
    pub address: Option<String>,
    /// Additional content added by the person who shared this activity, applicable only when resharing an activity.
    pub annotation: Option<String>,
    /// If this activity is a crosspost from another system, this property specifies the ID of the original activity.
    #[serde(rename="crosspostSource")]
    pub crosspost_source: Option<String>,
    /// ETag of this response for caching purposes.
    pub etag: Option<String>,
    /// Latitude and longitude where this activity occurred. Format is latitude followed by longitude, space separated.
    pub geocode: Option<String>,
    /// The ID of this activity.
    pub id: Option<String>,
    /// Identifies this resource as an activity. Value: "plus#activity".
    pub kind: Option<String>,
    /// The location where this activity occurred.
    pub location: Option<Place>,
    /// The object of this activity.
    pub object: Option<ActivityObject>,
    /// ID of the place where this activity occurred.
    #[serde(rename="placeId")]
    pub place_id: Option<String>,
    /// Name of the place where this activity occurred.
    #[serde(rename="placeName")]
    pub place_name: Option<String>,
    /// The service provider that initially published this activity.
    pub provider: Option<ActivityProvider>,
    /// The time at which this activity was initially published. Formatted as an RFC 3339 timestamp.
    pub published: Option<String>,
    /// Radius, in meters, of the region where this activity occurred, centered at the latitude and longitude identified in geocode.
    pub radius: Option<String>,
    /// Title of this activity.
    pub title: Option<String>,
    /// The time at which this activity was last updated. Formatted as an RFC 3339 timestamp.
    pub updated: Option<String>,
    /// The link to this activity.
    pub url: Option<String>,
    /// This activity's verb, which indicates the action that was performed. Possible values include, but are not limited to, the following values:  
    /// - "post" - Publish content to the stream. 
    /// - "share" - Reshare an activity.
    pub verb: Option<String>,
}

impl client::ResponseResult for Activity {}


/// There is no detailed description.
/// 
/// # Activities
/// 
/// This type is used in activities, which are methods you may call on this type or where this type is involved in. 
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
/// 
/// * [list activities](ActivityListCall) (response)
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ActivityFeed {
    /// ETag of this response for caching purposes.
    pub etag: Option<String>,
    /// The ID of this collection of activities. Deprecated.
    pub id: Option<String>,
    /// The activities in this page of results.
    pub items: Option<Vec<Activity>>,
    /// Identifies this resource as a collection of activities. Value: "plus#activityFeed".
    pub kind: Option<String>,
    /// Link to the next page of activities.
    #[serde(rename="nextLink")]
    pub next_link: Option<String>,
    /// The continuation token, which is used to page through large result sets. Provide this value in a subsequent request to return the next page of results.
    #[serde(rename="nextPageToken")]
    pub next_page_token: Option<String>,
    /// Link to this activity resource.
    #[serde(rename="selfLink")]
    pub self_link: Option<String>,
    /// The title of this collection of activities, which is a truncated portion of the content.
    pub title: Option<String>,
    /// The time at which this collection of activities was last updated. Formatted as an RFC 3339 timestamp.
    pub updated: Option<String>,
}

impl client::ResponseResult for ActivityFeed {}


/// There is no detailed description.
/// 
/// # Activities
/// 
/// This type is used in activities, which are methods you may call on this type or where this type is involved in. 
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
/// 
/// * [list audiences](AudienceListCall) (none)
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Audience {
    /// ETag of this response for caching purposes.
    pub etag: Option<String>,
    /// The access control list entry.
    pub item: Option<PlusDomainsAclentryResource>,
    /// Identifies this resource as an audience. Value: "plus#audience".
    pub kind: Option<String>,
    /// The number of people in this circle. This only applies if entity_type is CIRCLE.
    #[serde(rename="memberCount")]
    pub member_count: Option<u32>,
    /// The circle members' visibility as chosen by the owner of the circle. This only applies for items with "item.type" equals "circle". Possible values are:  
    /// - "public" - Members are visible to the public. 
    /// - "limited" - Members are visible to a limited audience. 
    /// - "private" - Members are visible to the owner only.
    pub visibility: Option<String>,
}

impl client::Resource for Audience {}


/// There is no detailed description.
/// 
/// # Activities
/// 
/// This type is used in activities, which are methods you may call on this type or where this type is involved in. 
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
/// 
/// * [list audiences](AudienceListCall) (response)
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct AudiencesFeed {
    /// ETag of this response for caching purposes.
    pub etag: Option<String>,
    /// The audiences in this result.
    pub items: Option<Vec<Audience>>,
    /// Identifies this resource as a collection of audiences. Value: "plus#audienceFeed".
    pub kind: Option<String>,
    /// The continuation token, which is used to page through large result sets. Provide this value in a subsequent request to return the next page of results.
    #[serde(rename="nextPageToken")]
    pub next_page_token: Option<String>,
    /// The total number of ACL entries. The number of entries in this response may be smaller due to paging.
    #[serde(rename="totalItems")]
    pub total_items: Option<i32>,
}

impl client::ResponseResult for AudiencesFeed {}


/// There is no detailed description.
/// 
/// # Activities
/// 
/// This type is used in activities, which are methods you may call on this type or where this type is involved in. 
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
/// 
/// * [list circles](CircleListCall) (none)
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Circle {
    /// The description of this circle.
    pub description: Option<String>,
    /// The circle name.
    #[serde(rename="displayName")]
    pub display_name: Option<String>,
    /// ETag of this response for caching purposes.
    pub etag: Option<String>,
    /// The ID of the circle.
    pub id: Option<String>,
    /// Identifies this resource as a circle. Value: "plus#circle".
    pub kind: Option<String>,
    /// The people in this circle.
    pub people: Option<CirclePeople>,
    /// Link to this circle resource
    #[serde(rename="selfLink")]
    pub self_link: Option<String>,
}

impl client::Resource for Circle {}


/// There is no detailed description.
/// 
/// # Activities
/// 
/// This type is used in activities, which are methods you may call on this type or where this type is involved in. 
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
/// 
/// * [list circles](CircleListCall) (response)
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct CircleFeed {
    /// ETag of this response for caching purposes.
    pub etag: Option<String>,
    /// The circles in this page of results.
    pub items: Option<Vec<Circle>>,
    /// Identifies this resource as a collection of circles. Value: "plus#circleFeed".
    pub kind: Option<String>,
    /// Link to the next page of circles.
    #[serde(rename="nextLink")]
    pub next_link: Option<String>,
    /// The continuation token, which is used to page through large result sets. Provide this value in a subsequent request to return the next page of results.
    #[serde(rename="nextPageToken")]
    pub next_page_token: Option<String>,
    /// Link to this page of circles.
    #[serde(rename="selfLink")]
    pub self_link: Option<String>,
    /// The title of this list of resources.
    pub title: Option<String>,
    /// The total number of circles. The number of circles in this response may be smaller due to paging.
    #[serde(rename="totalItems")]
    pub total_items: Option<i32>,
}

impl client::ResponseResult for CircleFeed {}


/// There is no detailed description.
/// 
/// # Activities
/// 
/// This type is used in activities, which are methods you may call on this type or where this type is involved in. 
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
/// 
/// * [get comments](CommentGetCall) (response)
/// * [list comments](CommentListCall) (none)
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Comment {
    /// The person who posted this comment.
    pub actor: Option<CommentActor>,
    /// ETag of this response for caching purposes.
    pub etag: Option<String>,
    /// The ID of this comment.
    pub id: Option<String>,
    /// The activity this comment replied to.
    #[serde(rename="inReplyTo")]
    pub in_reply_to: Option<Vec<CommentInReplyTo>>,
    /// Identifies this resource as a comment. Value: "plus#comment".
    pub kind: Option<String>,
    /// The object of this comment.
    pub object: Option<CommentObject>,
    /// People who +1'd this comment.
    pub plusoners: Option<CommentPlusoners>,
    /// The time at which this comment was initially published. Formatted as an RFC 3339 timestamp.
    pub published: Option<String>,
    /// Link to this comment resource.
    #[serde(rename="selfLink")]
    pub self_link: Option<String>,
    /// The time at which this comment was last updated. Formatted as an RFC 3339 timestamp.
    pub updated: Option<String>,
    /// This comment's verb, indicating what action was performed. Possible values are:  
    /// - "post" - Publish content to the stream.
    pub verb: Option<String>,
}

impl client::Resource for Comment {}
impl client::ResponseResult for Comment {}


/// There is no detailed description.
/// 
/// # Activities
/// 
/// This type is used in activities, which are methods you may call on this type or where this type is involved in. 
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
/// 
/// * [list comments](CommentListCall) (response)
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct CommentFeed {
    /// ETag of this response for caching purposes.
    pub etag: Option<String>,
    /// The ID of this collection of comments.
    pub id: Option<String>,
    /// The comments in this page of results.
    pub items: Option<Vec<Comment>>,
    /// Identifies this resource as a collection of comments. Value: "plus#commentFeed".
    pub kind: Option<String>,
    /// Link to the next page of activities.
    #[serde(rename="nextLink")]
    pub next_link: Option<String>,
    /// The continuation token, which is used to page through large result sets. Provide this value in a subsequent request to return the next page of results.
    #[serde(rename="nextPageToken")]
    pub next_page_token: Option<String>,
    /// The title of this collection of comments.
    pub title: Option<String>,
    /// The time at which this collection of comments was last updated. Formatted as an RFC 3339 timestamp.
    pub updated: Option<String>,
}

impl client::ResponseResult for CommentFeed {}


/// There is no detailed description.
/// 
/// # Activities
/// 
/// This type is used in activities, which are methods you may call on this type or where this type is involved in. 
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
/// 
/// * [insert media](MediaInsertCall) (request|response)
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Media {
    /// The person who uploaded this media.
    pub author: Option<MediaAuthor>,
    /// The display name for this media.
    #[serde(rename="displayName")]
    pub display_name: Option<String>,
    /// ETag of this response for caching purposes.
    pub etag: Option<String>,
    /// Exif information of the media item.
    pub exif: Option<MediaExif>,
    /// The height in pixels of the original image.
    pub height: Option<u32>,
    /// ID of this media, which is generated by the API.
    pub id: Option<String>,
    /// The type of resource.
    pub kind: Option<String>,
    /// The time at which this media was originally created in UTC. Formatted as an RFC 3339 timestamp that matches this example: 2010-11-25T14:30:27.655Z
    #[serde(rename="mediaCreatedTime")]
    pub media_created_time: Option<String>,
    /// The URL of this photo or video's still image.
    #[serde(rename="mediaUrl")]
    pub media_url: Option<String>,
    /// The time at which this media was uploaded. Formatted as an RFC 3339 timestamp.
    pub published: Option<String>,
    /// The size in bytes of this video.
    #[serde(rename="sizeBytes")]
    pub size_bytes: Option<String>,
    /// The list of video streams for this video. There might be several different streams available for a single video, either Flash or MPEG, of various sizes
    pub streams: Option<Vec<Videostream>>,
    /// A description, or caption, for this media.
    pub summary: Option<String>,
    /// The time at which this media was last updated. This includes changes to media metadata. Formatted as an RFC 3339 timestamp.
    pub updated: Option<String>,
    /// The URL for the page that hosts this media.
    pub url: Option<String>,
    /// The duration in milliseconds of this video.
    #[serde(rename="videoDuration")]
    pub video_duration: Option<String>,
    /// The encoding status of this video. Possible values are:  
    /// - "UPLOADING" - Not all the video bytes have been received. 
    /// - "PENDING" - Video not yet processed. 
    /// - "FAILED" - Video processing failed. 
    /// - "READY" - A single video stream is playable. 
    /// - "FINAL" - All video streams are playable.
    #[serde(rename="videoStatus")]
    pub video_status: Option<String>,
    /// The width in pixels of the original image.
    pub width: Option<u32>,
}

impl client::RequestValue for Media {}
impl client::ResponseResult for Media {}


/// There is no detailed description.
/// 
/// # Activities
/// 
/// This type is used in activities, which are methods you may call on this type or where this type is involved in. 
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
/// 
/// * [list people](PeopleListCall) (response)
/// * [list by activity people](PeopleListByActivityCall) (response)
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct PeopleFeed {
    /// ETag of this response for caching purposes.
    pub etag: Option<String>,
    /// The people in this page of results. Each item includes the id, displayName, image, and url for the person. To retrieve additional profile data, see the people.get method.
    pub items: Option<Vec<Person>>,
    /// Identifies this resource as a collection of people. Value: "plus#peopleFeed".
    pub kind: Option<String>,
    /// The continuation token, which is used to page through large result sets. Provide this value in a subsequent request to return the next page of results.
    #[serde(rename="nextPageToken")]
    pub next_page_token: Option<String>,
    /// Link to this resource.
    #[serde(rename="selfLink")]
    pub self_link: Option<String>,
    /// The title of this collection of people.
    pub title: Option<String>,
    /// The total number of people available in this list. The number of people in a response might be smaller due to paging. This might not be set for all collections.
    #[serde(rename="totalItems")]
    pub total_items: Option<i32>,
}

impl client::ResponseResult for PeopleFeed {}


/// There is no detailed description.
/// 
/// # Activities
/// 
/// This type is used in activities, which are methods you may call on this type or where this type is involved in. 
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
/// 
/// * [get people](PeopleGetCall) (response)
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Person {
    /// A short biography for this person.
    #[serde(rename="aboutMe")]
    pub about_me: Option<String>,
    /// The person's date of birth, represented as YYYY-MM-DD.
    pub birthday: Option<String>,
    /// The "bragging rights" line of this person.
    #[serde(rename="braggingRights")]
    pub bragging_rights: Option<String>,
    /// For followers who are visible, the number of people who have added this person or page to a circle.
    #[serde(rename="circledByCount")]
    pub circled_by_count: Option<i32>,
    /// The cover photo content.
    pub cover: Option<PersonCover>,
    /// (this field is not currently used)
    #[serde(rename="currentLocation")]
    pub current_location: Option<String>,
    /// The name of this person, which is suitable for display.
    #[serde(rename="displayName")]
    pub display_name: Option<String>,
    /// The hosted domain name for the user's Google Apps account. For instance, example.com. The plus.profile.emails.read or email scope is needed to get this domain name.
    pub domain: Option<String>,
    /// A list of email addresses that this person has, including their Google account email address, and the public verified email addresses on their Google+ profile. The plus.profile.emails.read scope is needed to retrieve these email addresses, or the email scope can be used to retrieve just the Google account email address.
    pub emails: Option<Vec<PersonEmails>>,
    /// ETag of this response for caching purposes.
    pub etag: Option<String>,
    /// The person's gender. Possible values include, but are not limited to, the following values:  
    /// - "male" - Male gender. 
    /// - "female" - Female gender. 
    /// - "other" - Other.
    pub gender: Option<String>,
    /// The ID of this person.
    pub id: Option<String>,
    /// The representation of the person's profile photo.
    pub image: Option<PersonImage>,
    /// Whether this user has signed up for Google+.
    #[serde(rename="isPlusUser")]
    pub is_plus_user: Option<bool>,
    /// Identifies this resource as a person. Value: "plus#person".
    pub kind: Option<String>,
    /// An object representation of the individual components of a person's name.
    pub name: Option<PersonName>,
    /// The nickname of this person.
    pub nickname: Option<String>,
    /// Type of person within Google+. Possible values include, but are not limited to, the following values:  
    /// - "person" - represents an actual person. 
    /// - "page" - represents a page.
    #[serde(rename="objectType")]
    pub object_type: Option<String>,
    /// The occupation of this person.
    pub occupation: Option<String>,
    /// A list of current or past organizations with which this person is associated.
    pub organizations: Option<Vec<PersonOrganizations>>,
    /// A list of places where this person has lived.
    #[serde(rename="placesLived")]
    pub places_lived: Option<Vec<PersonPlacesLived>>,
    /// If a Google+ Page, the number of people who have +1'd this page.
    #[serde(rename="plusOneCount")]
    pub plus_one_count: Option<i32>,
    /// The person's relationship status. Possible values include, but are not limited to, the following values:  
    /// - "single" - Person is single. 
    /// - "in_a_relationship" - Person is in a relationship. 
    /// - "engaged" - Person is engaged. 
    /// - "married" - Person is married. 
    /// - "its_complicated" - The relationship is complicated. 
    /// - "open_relationship" - Person is in an open relationship. 
    /// - "widowed" - Person is widowed. 
    /// - "in_domestic_partnership" - Person is in a domestic partnership. 
    /// - "in_civil_union" - Person is in a civil union.
    #[serde(rename="relationshipStatus")]
    pub relationship_status: Option<String>,
    /// The person's skills.
    pub skills: Option<String>,
    /// The brief description (tagline) of this person.
    pub tagline: Option<String>,
    /// The URL of this person's profile.
    pub url: Option<String>,
    /// A list of URLs for this person.
    pub urls: Option<Vec<PersonUrls>>,
    /// Whether the person or Google+ Page has been verified.
    pub verified: Option<bool>,
}

impl client::ResponseResult for Person {}


/// There is no detailed description.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Place {
    /// The physical address of the place.
    pub address: Option<PlaceAddress>,
    /// The display name of the place.
    #[serde(rename="displayName")]
    pub display_name: Option<String>,
    /// The id of the place.
    pub id: Option<String>,
    /// Identifies this resource as a place. Value: "plus#place".
    pub kind: Option<String>,
    /// The position of the place.
    pub position: Option<PlacePosition>,
}

impl client::Part for Place {}


/// There is no detailed description.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct PlusDomainsAclentryResource {
    /// A descriptive name for this entry. Suitable for display.
    #[serde(rename="displayName")]
    pub display_name: Option<String>,
    /// The ID of the entry. For entries of type "person" or "circle", this is the ID of the resource. For other types, this property is not set.
    pub id: Option<String>,
    /// The type of entry describing to whom access is granted. Possible values are:  
    /// - "person" - Access to an individual. 
    /// - "circle" - Access to members of a circle. 
    /// - "myCircles" - Access to members of all the person's circles. 
    /// - "extendedCircles" - Access to members of all the person's circles, plus all of the people in their circles. 
    /// - "domain" - Access to members of the person's Google Apps domain. 
    /// - "public" - Access to anyone on the web.
    #[serde(rename="type")]
    pub type_: Option<String>,
}

impl client::Part for PlusDomainsAclentryResource {}


/// There is no detailed description.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Videostream {
    /// The height, in pixels, of the video resource.
    pub height: Option<i32>,
    /// MIME type of the video stream.
    #[serde(rename="type")]
    pub type_: Option<String>,
    /// URL of the video stream.
    pub url: Option<String>,
    /// The width, in pixels, of the video resource.
    pub width: Option<i32>,
}

impl client::Part for Videostream {}


/// The person who performed this activity.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ActivityActor {
    /// Actor info specific to particular clients.
    #[serde(rename="clientSpecificActorInfo")]
    pub client_specific_actor_info: Option<ActivityActorClientSpecificActorInfo>,
    /// The name of the actor, suitable for display.
    #[serde(rename="displayName")]
    pub display_name: Option<String>,
    /// The ID of the actor's Person resource.
    pub id: Option<String>,
    /// The image representation of the actor.
    pub image: Option<ActivityActorImage>,
    /// An object representation of the individual components of name.
    pub name: Option<ActivityActorName>,
    /// The link to the actor's Google profile.
    pub url: Option<String>,
    /// Verification status of actor.
    pub verification: Option<ActivityActorVerification>,
}

impl client::NestedType for ActivityActor {}
impl client::Part for ActivityActor {}


/// Actor info specific to particular clients.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ActivityActorClientSpecificActorInfo {
    /// Actor info specific to YouTube clients.
    #[serde(rename="youtubeActorInfo")]
    pub youtube_actor_info: Option<ActivityActorClientSpecificActorInfoYoutubeActorInfo>,
}

impl client::NestedType for ActivityActorClientSpecificActorInfo {}
impl client::Part for ActivityActorClientSpecificActorInfo {}


/// Actor info specific to YouTube clients.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ActivityActorClientSpecificActorInfoYoutubeActorInfo {
    /// ID of the YouTube channel owned by the Actor.
    #[serde(rename="channelId")]
    pub channel_id: Option<String>,
}

impl client::NestedType for ActivityActorClientSpecificActorInfoYoutubeActorInfo {}
impl client::Part for ActivityActorClientSpecificActorInfoYoutubeActorInfo {}


/// The image representation of the actor.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ActivityActorImage {
    /// The URL of the actor's profile photo. To resize the image and crop it to a square, append the query string ?sz=x, where x is the dimension in pixels of each side.
    pub url: Option<String>,
}

impl client::NestedType for ActivityActorImage {}
impl client::Part for ActivityActorImage {}


/// An object representation of the individual components of name.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ActivityActorName {
    /// The family name ("last name") of the actor.
    #[serde(rename="familyName")]
    pub family_name: Option<String>,
    /// The given name ("first name") of the actor.
    #[serde(rename="givenName")]
    pub given_name: Option<String>,
}

impl client::NestedType for ActivityActorName {}
impl client::Part for ActivityActorName {}


/// Verification status of actor.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ActivityActorVerification {
    /// Verification for one-time or manual processes.
    #[serde(rename="adHocVerified")]
    pub ad_hoc_verified: Option<String>,
}

impl client::NestedType for ActivityActorVerification {}
impl client::Part for ActivityActorVerification {}


/// The object of this activity.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ActivityObject {
    /// If this activity's object is itself another activity, such as when a person reshares an activity, this property specifies the original activity's actor.
    pub actor: Option<ActivityObjectActor>,
    /// The media objects attached to this activity.
    pub attachments: Option<Vec<ActivityObjectAttachments>>,
    /// The HTML-formatted content, which is suitable for display.
    pub content: Option<String>,
    /// The ID of the object. When resharing an activity, this is the ID of the activity that is being reshared.
    pub id: Option<String>,
    /// The type of the object. Possible values include, but are not limited to, the following values:  
    /// - "note" - Textual content. 
    /// - "activity" - A Google+ activity.
    #[serde(rename="objectType")]
    pub object_type: Option<String>,
    /// The content (text) as provided by the author, which is stored without any HTML formatting. When creating or updating an activity, this value must be supplied as plain text in the request.
    #[serde(rename="originalContent")]
    pub original_content: Option<String>,
    /// People who +1'd this activity.
    pub plusoners: Option<ActivityObjectPlusoners>,
    /// Comments in reply to this activity.
    pub replies: Option<ActivityObjectReplies>,
    /// People who reshared this activity.
    pub resharers: Option<ActivityObjectResharers>,
    /// Status of the activity as seen by the viewer.
    #[serde(rename="statusForViewer")]
    pub status_for_viewer: Option<ActivityObjectStatusForViewer>,
    /// The URL that points to the linked resource.
    pub url: Option<String>,
}

impl client::NestedType for ActivityObject {}
impl client::Part for ActivityObject {}


/// If this activity's object is itself another activity, such as when a person reshares an activity, this property specifies the original activity's actor.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ActivityObjectActor {
    /// Actor info specific to particular clients.
    #[serde(rename="clientSpecificActorInfo")]
    pub client_specific_actor_info: Option<ActivityObjectActorClientSpecificActorInfo>,
    /// The original actor's name, which is suitable for display.
    #[serde(rename="displayName")]
    pub display_name: Option<String>,
    /// ID of the original actor.
    pub id: Option<String>,
    /// The image representation of the original actor.
    pub image: Option<ActivityObjectActorImage>,
    /// A link to the original actor's Google profile.
    pub url: Option<String>,
    /// Verification status of actor.
    pub verification: Option<ActivityObjectActorVerification>,
}

impl client::NestedType for ActivityObjectActor {}
impl client::Part for ActivityObjectActor {}


/// Actor info specific to particular clients.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ActivityObjectActorClientSpecificActorInfo {
    /// Actor info specific to YouTube clients.
    #[serde(rename="youtubeActorInfo")]
    pub youtube_actor_info: Option<ActivityObjectActorClientSpecificActorInfoYoutubeActorInfo>,
}

impl client::NestedType for ActivityObjectActorClientSpecificActorInfo {}
impl client::Part for ActivityObjectActorClientSpecificActorInfo {}


/// Actor info specific to YouTube clients.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ActivityObjectActorClientSpecificActorInfoYoutubeActorInfo {
    /// ID of the YouTube channel owned by the Actor.
    #[serde(rename="channelId")]
    pub channel_id: Option<String>,
}

impl client::NestedType for ActivityObjectActorClientSpecificActorInfoYoutubeActorInfo {}
impl client::Part for ActivityObjectActorClientSpecificActorInfoYoutubeActorInfo {}


/// The image representation of the original actor.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ActivityObjectActorImage {
    /// A URL that points to a thumbnail photo of the original actor.
    pub url: Option<String>,
}

impl client::NestedType for ActivityObjectActorImage {}
impl client::Part for ActivityObjectActorImage {}


/// Verification status of actor.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ActivityObjectActorVerification {
    /// Verification for one-time or manual processes.
    #[serde(rename="adHocVerified")]
    pub ad_hoc_verified: Option<String>,
}

impl client::NestedType for ActivityObjectActorVerification {}
impl client::Part for ActivityObjectActorVerification {}


/// The media objects attached to this activity.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ActivityObjectAttachments {
    /// If the attachment is an article, this property contains a snippet of text from the article. It can also include descriptions for other types.
    pub content: Option<String>,
    /// The title of the attachment, such as a photo caption or an article title.
    #[serde(rename="displayName")]
    pub display_name: Option<String>,
    /// If the attachment is a video, the embeddable link.
    pub embed: Option<ActivityObjectAttachmentsEmbed>,
    /// The full image URL for photo attachments.
    #[serde(rename="fullImage")]
    pub full_image: Option<ActivityObjectAttachmentsFullImage>,
    /// The ID of the attachment.
    pub id: Option<String>,
    /// The preview image for photos or videos.
    pub image: Option<ActivityObjectAttachmentsImage>,
    /// The type of media object. Possible values include, but are not limited to, the following values:  
    /// - "photo" - A photo. 
    /// - "album" - A photo album. 
    /// - "video" - A video. 
    /// - "article" - An article, specified by a link.
    #[serde(rename="objectType")]
    pub object_type: Option<String>,
    /// When previewing, these are the optional thumbnails for the post. When posting an article, choose one by setting the attachment.image.url property. If you don't choose one, one will be chosen for you.
    #[serde(rename="previewThumbnails")]
    pub preview_thumbnails: Option<Vec<ActivityObjectAttachmentsPreviewThumbnails>>,
    /// If the attachment is an album, this property is a list of potential additional thumbnails from the album.
    pub thumbnails: Option<Vec<ActivityObjectAttachmentsThumbnails>>,
    /// The link to the attachment, which should be of type text/html.
    pub url: Option<String>,
}

impl client::NestedType for ActivityObjectAttachments {}
impl client::Part for ActivityObjectAttachments {}


/// If the attachment is a video, the embeddable link.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ActivityObjectAttachmentsEmbed {
    /// Media type of the link.
    #[serde(rename="type")]
    pub type_: Option<String>,
    /// URL of the link.
    pub url: Option<String>,
}

impl client::NestedType for ActivityObjectAttachmentsEmbed {}
impl client::Part for ActivityObjectAttachmentsEmbed {}


/// The full image URL for photo attachments.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ActivityObjectAttachmentsFullImage {
    /// The height, in pixels, of the linked resource.
    pub height: Option<u32>,
    /// Media type of the link.
    #[serde(rename="type")]
    pub type_: Option<String>,
    /// URL of the image.
    pub url: Option<String>,
    /// The width, in pixels, of the linked resource.
    pub width: Option<u32>,
}

impl client::NestedType for ActivityObjectAttachmentsFullImage {}
impl client::Part for ActivityObjectAttachmentsFullImage {}


/// The preview image for photos or videos.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ActivityObjectAttachmentsImage {
    /// The height, in pixels, of the linked resource.
    pub height: Option<u32>,
    /// Media type of the link.
    #[serde(rename="type")]
    pub type_: Option<String>,
    /// Image URL.
    pub url: Option<String>,
    /// The width, in pixels, of the linked resource.
    pub width: Option<u32>,
}

impl client::NestedType for ActivityObjectAttachmentsImage {}
impl client::Part for ActivityObjectAttachmentsImage {}


/// When previewing, these are the optional thumbnails for the post. When posting an article, choose one by setting the attachment.image.url property. If you don't choose one, one will be chosen for you.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ActivityObjectAttachmentsPreviewThumbnails {
    /// URL of the thumbnail image.
    pub url: Option<String>,
}

impl client::NestedType for ActivityObjectAttachmentsPreviewThumbnails {}
impl client::Part for ActivityObjectAttachmentsPreviewThumbnails {}


/// If the attachment is an album, this property is a list of potential additional thumbnails from the album.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ActivityObjectAttachmentsThumbnails {
    /// Potential name of the thumbnail.
    pub description: Option<String>,
    /// Image resource.
    pub image: Option<ActivityObjectAttachmentsThumbnailsImage>,
    /// URL of the webpage containing the image.
    pub url: Option<String>,
}

impl client::NestedType for ActivityObjectAttachmentsThumbnails {}
impl client::Part for ActivityObjectAttachmentsThumbnails {}


/// Image resource.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ActivityObjectAttachmentsThumbnailsImage {
    /// The height, in pixels, of the linked resource.
    pub height: Option<u32>,
    /// Media type of the link.
    #[serde(rename="type")]
    pub type_: Option<String>,
    /// Image url.
    pub url: Option<String>,
    /// The width, in pixels, of the linked resource.
    pub width: Option<u32>,
}

impl client::NestedType for ActivityObjectAttachmentsThumbnailsImage {}
impl client::Part for ActivityObjectAttachmentsThumbnailsImage {}


/// People who +1'd this activity.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ActivityObjectPlusoners {
    /// The URL for the collection of people who +1'd this activity.
    #[serde(rename="selfLink")]
    pub self_link: Option<String>,
    /// Total number of people who +1'd this activity.
    #[serde(rename="totalItems")]
    pub total_items: Option<u32>,
}

impl client::NestedType for ActivityObjectPlusoners {}
impl client::Part for ActivityObjectPlusoners {}


/// Comments in reply to this activity.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ActivityObjectReplies {
    /// The URL for the collection of comments in reply to this activity.
    #[serde(rename="selfLink")]
    pub self_link: Option<String>,
    /// Total number of comments on this activity.
    #[serde(rename="totalItems")]
    pub total_items: Option<u32>,
}

impl client::NestedType for ActivityObjectReplies {}
impl client::Part for ActivityObjectReplies {}


/// People who reshared this activity.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ActivityObjectResharers {
    /// The URL for the collection of resharers.
    #[serde(rename="selfLink")]
    pub self_link: Option<String>,
    /// Total number of people who reshared this activity.
    #[serde(rename="totalItems")]
    pub total_items: Option<u32>,
}

impl client::NestedType for ActivityObjectResharers {}
impl client::Part for ActivityObjectResharers {}


/// Status of the activity as seen by the viewer.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ActivityObjectStatusForViewer {
    /// Whether the viewer can comment on the activity.
    #[serde(rename="canComment")]
    pub can_comment: Option<bool>,
    /// Whether the viewer can +1 the activity.
    #[serde(rename="canPlusone")]
    pub can_plusone: Option<bool>,
    /// Whether the viewer can edit or delete the activity.
    #[serde(rename="canUpdate")]
    pub can_update: Option<bool>,
    /// Whether the viewer has +1'd the activity.
    #[serde(rename="isPlusOned")]
    pub is_plus_oned: Option<bool>,
    /// Whether reshares are disabled for the activity.
    #[serde(rename="resharingDisabled")]
    pub resharing_disabled: Option<bool>,
}

impl client::NestedType for ActivityObjectStatusForViewer {}
impl client::Part for ActivityObjectStatusForViewer {}


/// The service provider that initially published this activity.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ActivityProvider {
    /// Name of the service provider.
    pub title: Option<String>,
}

impl client::NestedType for ActivityProvider {}
impl client::Part for ActivityProvider {}


/// The people in this circle.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct CirclePeople {
    /// The total number of people in this circle.
    #[serde(rename="totalItems")]
    pub total_items: Option<u32>,
}

impl client::NestedType for CirclePeople {}
impl client::Part for CirclePeople {}


/// The person who posted this comment.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct CommentActor {
    /// Actor info specific to particular clients.
    #[serde(rename="clientSpecificActorInfo")]
    pub client_specific_actor_info: Option<CommentActorClientSpecificActorInfo>,
    /// The name of this actor, suitable for display.
    #[serde(rename="displayName")]
    pub display_name: Option<String>,
    /// The ID of the actor.
    pub id: Option<String>,
    /// The image representation of this actor.
    pub image: Option<CommentActorImage>,
    /// A link to the Person resource for this actor.
    pub url: Option<String>,
    /// Verification status of actor.
    pub verification: Option<CommentActorVerification>,
}

impl client::NestedType for CommentActor {}
impl client::Part for CommentActor {}


/// Actor info specific to particular clients.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct CommentActorClientSpecificActorInfo {
    /// Actor info specific to YouTube clients.
    #[serde(rename="youtubeActorInfo")]
    pub youtube_actor_info: Option<CommentActorClientSpecificActorInfoYoutubeActorInfo>,
}

impl client::NestedType for CommentActorClientSpecificActorInfo {}
impl client::Part for CommentActorClientSpecificActorInfo {}


/// Actor info specific to YouTube clients.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct CommentActorClientSpecificActorInfoYoutubeActorInfo {
    /// ID of the YouTube channel owned by the Actor.
    #[serde(rename="channelId")]
    pub channel_id: Option<String>,
}

impl client::NestedType for CommentActorClientSpecificActorInfoYoutubeActorInfo {}
impl client::Part for CommentActorClientSpecificActorInfoYoutubeActorInfo {}


/// The image representation of this actor.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct CommentActorImage {
    /// The URL of the actor's profile photo. To resize the image and crop it to a square, append the query string ?sz=x, where x is the dimension in pixels of each side.
    pub url: Option<String>,
}

impl client::NestedType for CommentActorImage {}
impl client::Part for CommentActorImage {}


/// Verification status of actor.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct CommentActorVerification {
    /// Verification for one-time or manual processes.
    #[serde(rename="adHocVerified")]
    pub ad_hoc_verified: Option<String>,
}

impl client::NestedType for CommentActorVerification {}
impl client::Part for CommentActorVerification {}


/// The activity this comment replied to.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct CommentInReplyTo {
    /// The ID of the activity.
    pub id: Option<String>,
    /// The URL of the activity.
    pub url: Option<String>,
}

impl client::NestedType for CommentInReplyTo {}
impl client::Part for CommentInReplyTo {}


/// The object of this comment.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct CommentObject {
    /// The HTML-formatted content, suitable for display.
    pub content: Option<String>,
    /// The object type of this comment. Possible values are:  
    /// - "comment" - A comment in reply to an activity.
    #[serde(rename="objectType")]
    pub object_type: Option<String>,
    /// The content (text) as provided by the author, stored without any HTML formatting. When creating or updating a comment, this value must be supplied as plain text in the request.
    #[serde(rename="originalContent")]
    pub original_content: Option<String>,
}

impl client::NestedType for CommentObject {}
impl client::Part for CommentObject {}


/// People who +1'd this comment.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct CommentPlusoners {
    /// Total number of people who +1'd this comment.
    #[serde(rename="totalItems")]
    pub total_items: Option<u32>,
}

impl client::NestedType for CommentPlusoners {}
impl client::Part for CommentPlusoners {}


/// The person who uploaded this media.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct MediaAuthor {
    /// The author's name.
    #[serde(rename="displayName")]
    pub display_name: Option<String>,
    /// ID of the author.
    pub id: Option<String>,
    /// The author's Google profile image.
    pub image: Option<MediaAuthorImage>,
    /// A link to the author's Google profile.
    pub url: Option<String>,
}

impl client::NestedType for MediaAuthor {}
impl client::Part for MediaAuthor {}


/// The author's Google profile image.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct MediaAuthorImage {
    /// The URL of the author's profile photo. To resize the image and crop it to a square, append the query string ?sz=x, where x is the dimension in pixels of each side.
    pub url: Option<String>,
}

impl client::NestedType for MediaAuthorImage {}
impl client::Part for MediaAuthorImage {}


/// Exif information of the media item.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct MediaExif {
    /// The time the media was captured. Formatted as an RFC 3339 timestamp.
    pub time: Option<String>,
}

impl client::NestedType for MediaExif {}
impl client::Part for MediaExif {}


/// The cover photo content.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct PersonCover {
    /// Extra information about the cover photo.
    #[serde(rename="coverInfo")]
    pub cover_info: Option<PersonCoverCoverInfo>,
    /// The person's primary cover image.
    #[serde(rename="coverPhoto")]
    pub cover_photo: Option<PersonCoverCoverPhoto>,
    /// The layout of the cover art. Possible values include, but are not limited to, the following values:  
    /// - "banner" - One large image banner.
    pub layout: Option<String>,
}

impl client::NestedType for PersonCover {}
impl client::Part for PersonCover {}


/// Extra information about the cover photo.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct PersonCoverCoverInfo {
    /// The difference between the left position of the cover image and the actual displayed cover image. Only valid for banner layout.
    #[serde(rename="leftImageOffset")]
    pub left_image_offset: Option<i32>,
    /// The difference between the top position of the cover image and the actual displayed cover image. Only valid for banner layout.
    #[serde(rename="topImageOffset")]
    pub top_image_offset: Option<i32>,
}

impl client::NestedType for PersonCoverCoverInfo {}
impl client::Part for PersonCoverCoverInfo {}


/// The person's primary cover image.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct PersonCoverCoverPhoto {
    /// The height of the image.
    pub height: Option<i32>,
    /// The URL of the image.
    pub url: Option<String>,
    /// The width of the image.
    pub width: Option<i32>,
}

impl client::NestedType for PersonCoverCoverPhoto {}
impl client::Part for PersonCoverCoverPhoto {}


/// A list of email addresses that this person has, including their Google account email address, and the public verified email addresses on their Google+ profile. The plus.profile.emails.read scope is needed to retrieve these email addresses, or the email scope can be used to retrieve just the Google account email address.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct PersonEmails {
    /// The type of address. Possible values include, but are not limited to, the following values:  
    /// - "account" - Google account email address. 
    /// - "home" - Home email address. 
    /// - "work" - Work email address. 
    /// - "other" - Other.
    #[serde(rename="type")]
    pub type_: Option<String>,
    /// The email address.
    pub value: Option<String>,
}

impl client::NestedType for PersonEmails {}
impl client::Part for PersonEmails {}


/// The representation of the person's profile photo.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct PersonImage {
    /// Whether the person's profile photo is the default one
    #[serde(rename="isDefault")]
    pub is_default: Option<bool>,
    /// The URL of the person's profile photo. To resize the image and crop it to a square, append the query string ?sz=x, where x is the dimension in pixels of each side.
    pub url: Option<String>,
}

impl client::NestedType for PersonImage {}
impl client::Part for PersonImage {}


/// An object representation of the individual components of a person's name.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct PersonName {
    /// The family name (last name) of this person.
    #[serde(rename="familyName")]
    pub family_name: Option<String>,
    /// The full name of this person, including middle names, suffixes, etc.
    pub formatted: Option<String>,
    /// The given name (first name) of this person.
    #[serde(rename="givenName")]
    pub given_name: Option<String>,
    /// The honorific prefixes (such as "Dr." or "Mrs.") for this person.
    #[serde(rename="honorificPrefix")]
    pub honorific_prefix: Option<String>,
    /// The honorific suffixes (such as "Jr.") for this person.
    #[serde(rename="honorificSuffix")]
    pub honorific_suffix: Option<String>,
    /// The middle name of this person.
    #[serde(rename="middleName")]
    pub middle_name: Option<String>,
}

impl client::NestedType for PersonName {}
impl client::Part for PersonName {}


/// A list of current or past organizations with which this person is associated.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct PersonOrganizations {
    /// The department within the organization. Deprecated.
    pub department: Option<String>,
    /// A short description of the person's role in this organization. Deprecated.
    pub description: Option<String>,
    /// The date that the person left this organization.
    #[serde(rename="endDate")]
    pub end_date: Option<String>,
    /// The location of this organization. Deprecated.
    pub location: Option<String>,
    /// The name of the organization.
    pub name: Option<String>,
    /// If "true", indicates this organization is the person's primary one, which is typically interpreted as the current one.
    pub primary: Option<bool>,
    /// The date that the person joined this organization.
    #[serde(rename="startDate")]
    pub start_date: Option<String>,
    /// The person's job title or role within the organization.
    pub title: Option<String>,
    /// The type of organization. Possible values include, but are not limited to, the following values:  
    /// - "work" - Work. 
    /// - "school" - School.
    #[serde(rename="type")]
    pub type_: Option<String>,
}

impl client::NestedType for PersonOrganizations {}
impl client::Part for PersonOrganizations {}


/// A list of places where this person has lived.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct PersonPlacesLived {
    /// If "true", this place of residence is this person's primary residence.
    pub primary: Option<bool>,
    /// A place where this person has lived. For example: "Seattle, WA", "Near Toronto".
    pub value: Option<String>,
}

impl client::NestedType for PersonPlacesLived {}
impl client::Part for PersonPlacesLived {}


/// A list of URLs for this person.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct PersonUrls {
    /// The label of the URL.
    pub label: Option<String>,
    /// The type of URL. Possible values include, but are not limited to, the following values:  
    /// - "otherProfile" - URL for another profile. 
    /// - "contributor" - URL to a site for which this person is a contributor. 
    /// - "website" - URL for this Google+ Page's primary website. 
    /// - "other" - Other URL.
    #[serde(rename="type")]
    pub type_: Option<String>,
    /// The URL value.
    pub value: Option<String>,
}

impl client::NestedType for PersonUrls {}
impl client::Part for PersonUrls {}


/// The physical address of the place.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct PlaceAddress {
    /// The formatted address for display.
    pub formatted: Option<String>,
}

impl client::NestedType for PlaceAddress {}
impl client::Part for PlaceAddress {}


/// The position of the place.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct PlacePosition {
    /// The latitude of this position.
    pub latitude: Option<f64>,
    /// The longitude of this position.
    pub longitude: Option<f64>,
}

impl client::NestedType for PlacePosition {}
impl client::Part for PlacePosition {}



// ###################
// MethodBuilders ###
// #################

/// A builder providing access to all methods supported on *activity* resources.
/// It is not used directly, but through the `PlusDomains` hub.
///
/// # Example
///
/// Instantiate a resource builder
///
/// ```test_harness,no_run
/// extern crate hyper;
/// extern crate hyper_rustls;
/// extern crate yup_oauth2 as oauth2;
/// extern crate google_plusdomains1 as plusdomains1;
/// 
/// # async fn dox() {
/// use std::default::Default;
/// use oauth2;
/// use plusdomains1::PlusDomains;
/// 
/// let secret: oauth2::ApplicationSecret = Default::default();
/// let auth = yup_oauth2::InstalledFlowAuthenticator::builder(
///         secret,
///         yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
///     ).build().await.unwrap();
/// let mut hub = PlusDomains::new(hyper::Client::builder().build(hyper_rustls::HttpsConnector::with_native_roots()), auth);
/// // Usually you wouldn't bind this to a variable, but keep calling *CallBuilders*
/// // like `get(...)` and `list(...)`
/// // to build up your call.
/// let rb = hub.activities();
/// # }
/// ```
pub struct ActivityMethods<'a>
    where  {

    hub: &'a PlusDomains<>,
}

impl<'a> client::MethodsBuilder for ActivityMethods<'a> {}

impl<'a> ActivityMethods<'a> {
    
    /// Create a builder to help you perform the following task:
    ///
    /// Shut down. See https://developers.google.com/+/api-shutdown for more details.
    /// 
    /// # Arguments
    ///
    /// * `activityId` - The ID of the activity to get.
    pub fn get(&self, activity_id: &str) -> ActivityGetCall<'a> {
        ActivityGetCall {
            hub: self.hub,
            _activity_id: activity_id.to_string(),
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }
    
    /// Create a builder to help you perform the following task:
    ///
    /// Shut down. See https://developers.google.com/+/api-shutdown for more details.
    /// 
    /// # Arguments
    ///
    /// * `userId` - The ID of the user to get activities for. The special value "me" can be used to indicate the authenticated user.
    /// * `collection` - The collection of activities to list.
    pub fn list(&self, user_id: &str, collection: &str) -> ActivityListCall<'a> {
        ActivityListCall {
            hub: self.hub,
            _user_id: user_id.to_string(),
            _collection: collection.to_string(),
            _page_token: Default::default(),
            _max_results: Default::default(),
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }
}



/// A builder providing access to all methods supported on *audience* resources.
/// It is not used directly, but through the `PlusDomains` hub.
///
/// # Example
///
/// Instantiate a resource builder
///
/// ```test_harness,no_run
/// extern crate hyper;
/// extern crate hyper_rustls;
/// extern crate yup_oauth2 as oauth2;
/// extern crate google_plusdomains1 as plusdomains1;
/// 
/// # async fn dox() {
/// use std::default::Default;
/// use oauth2;
/// use plusdomains1::PlusDomains;
/// 
/// let secret: oauth2::ApplicationSecret = Default::default();
/// let auth = yup_oauth2::InstalledFlowAuthenticator::builder(
///         secret,
///         yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
///     ).build().await.unwrap();
/// let mut hub = PlusDomains::new(hyper::Client::builder().build(hyper_rustls::HttpsConnector::with_native_roots()), auth);
/// // Usually you wouldn't bind this to a variable, but keep calling *CallBuilders*
/// // like `list(...)`
/// // to build up your call.
/// let rb = hub.audiences();
/// # }
/// ```
pub struct AudienceMethods<'a>
    where  {

    hub: &'a PlusDomains<>,
}

impl<'a> client::MethodsBuilder for AudienceMethods<'a> {}

impl<'a> AudienceMethods<'a> {
    
    /// Create a builder to help you perform the following task:
    ///
    /// Shut down. See https://developers.google.com/+/api-shutdown for more details.
    /// 
    /// # Arguments
    ///
    /// * `userId` - The ID of the user to get audiences for. The special value "me" can be used to indicate the authenticated user.
    pub fn list(&self, user_id: &str) -> AudienceListCall<'a> {
        AudienceListCall {
            hub: self.hub,
            _user_id: user_id.to_string(),
            _page_token: Default::default(),
            _max_results: Default::default(),
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }
}



/// A builder providing access to all methods supported on *circle* resources.
/// It is not used directly, but through the `PlusDomains` hub.
///
/// # Example
///
/// Instantiate a resource builder
///
/// ```test_harness,no_run
/// extern crate hyper;
/// extern crate hyper_rustls;
/// extern crate yup_oauth2 as oauth2;
/// extern crate google_plusdomains1 as plusdomains1;
/// 
/// # async fn dox() {
/// use std::default::Default;
/// use oauth2;
/// use plusdomains1::PlusDomains;
/// 
/// let secret: oauth2::ApplicationSecret = Default::default();
/// let auth = yup_oauth2::InstalledFlowAuthenticator::builder(
///         secret,
///         yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
///     ).build().await.unwrap();
/// let mut hub = PlusDomains::new(hyper::Client::builder().build(hyper_rustls::HttpsConnector::with_native_roots()), auth);
/// // Usually you wouldn't bind this to a variable, but keep calling *CallBuilders*
/// // like `list(...)`
/// // to build up your call.
/// let rb = hub.circles();
/// # }
/// ```
pub struct CircleMethods<'a>
    where  {

    hub: &'a PlusDomains<>,
}

impl<'a> client::MethodsBuilder for CircleMethods<'a> {}

impl<'a> CircleMethods<'a> {
    
    /// Create a builder to help you perform the following task:
    ///
    /// Shut down. See https://developers.google.com/+/api-shutdown for more details.
    /// 
    /// # Arguments
    ///
    /// * `userId` - The ID of the user to get circles for. The special value "me" can be used to indicate the authenticated user.
    pub fn list(&self, user_id: &str) -> CircleListCall<'a> {
        CircleListCall {
            hub: self.hub,
            _user_id: user_id.to_string(),
            _page_token: Default::default(),
            _max_results: Default::default(),
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }
}



/// A builder providing access to all methods supported on *comment* resources.
/// It is not used directly, but through the `PlusDomains` hub.
///
/// # Example
///
/// Instantiate a resource builder
///
/// ```test_harness,no_run
/// extern crate hyper;
/// extern crate hyper_rustls;
/// extern crate yup_oauth2 as oauth2;
/// extern crate google_plusdomains1 as plusdomains1;
/// 
/// # async fn dox() {
/// use std::default::Default;
/// use oauth2;
/// use plusdomains1::PlusDomains;
/// 
/// let secret: oauth2::ApplicationSecret = Default::default();
/// let auth = yup_oauth2::InstalledFlowAuthenticator::builder(
///         secret,
///         yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
///     ).build().await.unwrap();
/// let mut hub = PlusDomains::new(hyper::Client::builder().build(hyper_rustls::HttpsConnector::with_native_roots()), auth);
/// // Usually you wouldn't bind this to a variable, but keep calling *CallBuilders*
/// // like `get(...)` and `list(...)`
/// // to build up your call.
/// let rb = hub.comments();
/// # }
/// ```
pub struct CommentMethods<'a>
    where  {

    hub: &'a PlusDomains<>,
}

impl<'a> client::MethodsBuilder for CommentMethods<'a> {}

impl<'a> CommentMethods<'a> {
    
    /// Create a builder to help you perform the following task:
    ///
    /// Shut down. See https://developers.google.com/+/api-shutdown for more details.
    /// 
    /// # Arguments
    ///
    /// * `commentId` - The ID of the comment to get.
    pub fn get(&self, comment_id: &str) -> CommentGetCall<'a> {
        CommentGetCall {
            hub: self.hub,
            _comment_id: comment_id.to_string(),
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }
    
    /// Create a builder to help you perform the following task:
    ///
    /// Shut down. See https://developers.google.com/+/api-shutdown for more details.
    /// 
    /// # Arguments
    ///
    /// * `activityId` - The ID of the activity to get comments for.
    pub fn list(&self, activity_id: &str) -> CommentListCall<'a> {
        CommentListCall {
            hub: self.hub,
            _activity_id: activity_id.to_string(),
            _sort_order: Default::default(),
            _page_token: Default::default(),
            _max_results: Default::default(),
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }
}



/// A builder providing access to all methods supported on *media* resources.
/// It is not used directly, but through the `PlusDomains` hub.
///
/// # Example
///
/// Instantiate a resource builder
///
/// ```test_harness,no_run
/// extern crate hyper;
/// extern crate hyper_rustls;
/// extern crate yup_oauth2 as oauth2;
/// extern crate google_plusdomains1 as plusdomains1;
/// 
/// # async fn dox() {
/// use std::default::Default;
/// use oauth2;
/// use plusdomains1::PlusDomains;
/// 
/// let secret: oauth2::ApplicationSecret = Default::default();
/// let auth = yup_oauth2::InstalledFlowAuthenticator::builder(
///         secret,
///         yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
///     ).build().await.unwrap();
/// let mut hub = PlusDomains::new(hyper::Client::builder().build(hyper_rustls::HttpsConnector::with_native_roots()), auth);
/// // Usually you wouldn't bind this to a variable, but keep calling *CallBuilders*
/// // like `insert(...)`
/// // to build up your call.
/// let rb = hub.media();
/// # }
/// ```
pub struct MediaMethods<'a>
    where  {

    hub: &'a PlusDomains<>,
}

impl<'a> client::MethodsBuilder for MediaMethods<'a> {}

impl<'a> MediaMethods<'a> {
    
    /// Create a builder to help you perform the following task:
    ///
    /// Shut down. See https://developers.google.com/+/api-shutdown for more details.
    /// 
    /// # Arguments
    ///
    /// * `request` - No description provided.
    /// * `userId` - The ID of the user to create the activity on behalf of.
    /// * `collection` - No description provided.
    pub fn insert(&self, request: Media, user_id: &str, collection: &str) -> MediaInsertCall<'a> {
        MediaInsertCall {
            hub: self.hub,
            _request: request,
            _user_id: user_id.to_string(),
            _collection: collection.to_string(),
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }
}



/// A builder providing access to all methods supported on *people* resources.
/// It is not used directly, but through the `PlusDomains` hub.
///
/// # Example
///
/// Instantiate a resource builder
///
/// ```test_harness,no_run
/// extern crate hyper;
/// extern crate hyper_rustls;
/// extern crate yup_oauth2 as oauth2;
/// extern crate google_plusdomains1 as plusdomains1;
/// 
/// # async fn dox() {
/// use std::default::Default;
/// use oauth2;
/// use plusdomains1::PlusDomains;
/// 
/// let secret: oauth2::ApplicationSecret = Default::default();
/// let auth = yup_oauth2::InstalledFlowAuthenticator::builder(
///         secret,
///         yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
///     ).build().await.unwrap();
/// let mut hub = PlusDomains::new(hyper::Client::builder().build(hyper_rustls::HttpsConnector::with_native_roots()), auth);
/// // Usually you wouldn't bind this to a variable, but keep calling *CallBuilders*
/// // like `get(...)`, `list(...)` and `list_by_activity(...)`
/// // to build up your call.
/// let rb = hub.people();
/// # }
/// ```
pub struct PeopleMethods<'a>
    where  {

    hub: &'a PlusDomains<>,
}

impl<'a> client::MethodsBuilder for PeopleMethods<'a> {}

impl<'a> PeopleMethods<'a> {
    
    /// Create a builder to help you perform the following task:
    ///
    /// Get a person's profile.
    /// 
    /// # Arguments
    ///
    /// * `userId` - The ID of the person to get the profile for. The special value "me" can be used to indicate the authenticated user.
    pub fn get(&self, user_id: &str) -> PeopleGetCall<'a> {
        PeopleGetCall {
            hub: self.hub,
            _user_id: user_id.to_string(),
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }
    
    /// Create a builder to help you perform the following task:
    ///
    /// List all of the people in the specified collection.
    /// 
    /// # Arguments
    ///
    /// * `userId` - Get the collection of people for the person identified. Use "me" to indicate the authenticated user.
    /// * `collection` - The collection of people to list.
    pub fn list(&self, user_id: &str, collection: &str) -> PeopleListCall<'a> {
        PeopleListCall {
            hub: self.hub,
            _user_id: user_id.to_string(),
            _collection: collection.to_string(),
            _page_token: Default::default(),
            _order_by: Default::default(),
            _max_results: Default::default(),
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }
    
    /// Create a builder to help you perform the following task:
    ///
    /// Shut down. See https://developers.google.com/+/api-shutdown for more details.
    /// 
    /// # Arguments
    ///
    /// * `activityId` - The ID of the activity to get the list of people for.
    /// * `collection` - The collection of people to list.
    pub fn list_by_activity(&self, activity_id: &str, collection: &str) -> PeopleListByActivityCall<'a> {
        PeopleListByActivityCall {
            hub: self.hub,
            _activity_id: activity_id.to_string(),
            _collection: collection.to_string(),
            _page_token: Default::default(),
            _max_results: Default::default(),
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }
}





// ###################
// CallBuilders   ###
// #################

/// Shut down. See https://developers.google.com/+/api-shutdown for more details.
///
/// A builder for the *get* method supported by a *activity* resource.
/// It is not used directly, but through a `ActivityMethods` instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate yup_oauth2 as oauth2;
/// # extern crate google_plusdomains1 as plusdomains1;
/// # async fn dox() {
/// # use std::default::Default;
/// # use oauth2;
/// # use plusdomains1::PlusDomains;
/// 
/// # let secret: oauth2::ApplicationSecret = Default::default();
/// # let auth = yup_oauth2::InstalledFlowAuthenticator::builder(
/// #         secret,
/// #         yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     ).build().await.unwrap();
/// # let mut hub = PlusDomains::new(hyper::Client::builder().build(hyper_rustls::HttpsConnector::with_native_roots()), auth);
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.activities().get("activityId")
///              .doit().await;
/// # }
/// ```
pub struct ActivityGetCall<'a>
    where  {

    hub: &'a PlusDomains<>,
    _activity_id: String,
    _delegate: Option<&'a mut dyn client::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeMap<String, ()>
}

impl<'a> client::CallBuilder for ActivityGetCall<'a> {}

impl<'a> ActivityGetCall<'a> {


    /// Perform the operation you have build so far.
    pub async fn doit(mut self) -> client::Result<(hyper::Response<hyper::body::Body>, Activity)> {
        use std::io::{Read, Seek};
        use hyper::header::{CONTENT_TYPE, CONTENT_LENGTH, AUTHORIZATION, USER_AGENT, LOCATION};
        use client::ToParts;
        let mut dd = client::DefaultDelegate;
        let mut dlg: &mut dyn client::Delegate = match self._delegate {
            Some(d) => d,
            None => &mut dd
        };
        dlg.begin(client::MethodInfo { id: "plusDomains.activities.get",
                               http_method: hyper::Method::GET });
        let mut params: Vec<(&str, String)> = Vec::with_capacity(3 + self._additional_params.len());
        params.push(("activityId", self._activity_id.to_string()));
        for &field in ["alt", "activityId"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(client::Error::FieldClash(field));
            }
        }
        for (name, value) in self._additional_params.iter() {
            params.push((&name, value.clone()));
        }

        params.push(("alt", "json".to_string()));

        let mut url = self.hub._base_url.clone() + "activities/{activityId}";
        if self._scopes.len() == 0 {
            self._scopes.insert(Scope::PluLogin.as_ref().to_string(), ());
        }

        for &(find_this, param_name) in [("{activityId}", "activityId")].iter() {
            let mut replace_with: Option<&str> = None;
            for &(name, ref value) in params.iter() {
                if name == param_name {
                    replace_with = Some(value);
                    break;
                }
            }
            url = url.replace(find_this, replace_with.expect("to find substitution value in params"));
        }
        {
            let mut indices_for_removal: Vec<usize> = Vec::with_capacity(1);
            for param_name in ["activityId"].iter() {
                if let Some(index) = params.iter().position(|t| &t.0 == param_name) {
                    indices_for_removal.push(index);
                }
            }
            for &index in indices_for_removal.iter() {
                params.remove(index);
            }
        }

        let url = url::Url::parse_with_params(&url, params).unwrap();



        loop {
            let token = match self.hub.auth.token(&self._scopes.keys().collect::<Vec<_>>()[..]).await {
                Ok(token) => token.clone(),
                Err(err) => {
                    match  dlg.token(&err) {
                        Some(token) => token,
                        None => {
                            dlg.finished(false);
                            return Err(client::Error::MissingToken(err))
                        }
                    }
                }
            };
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder().method(hyper::Method::GET).uri(url.clone().into_string())
                        .header(USER_AGENT, self.hub._user_agent.clone())                            .header(AUTHORIZATION, format!("Bearer {}", token.as_str()));


                        let request = req_builder
                        .body(hyper::body::Body::empty());

                client.request(request.unwrap()).await
                
            };

            match req_result {
                Err(err) => {
                    if let client::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d);
                        continue;
                    }
                    dlg.finished(false);
                    return Err(client::Error::HttpError(err))
                }
                Ok(mut res) => {
                    if !res.status().is_success() {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        let json_server_error = json::from_str::<client::JsonServerError>(&res_body_string).ok();
                        let server_error = json::from_str::<client::ServerError>(&res_body_string)
                            .or_else(|_| json::from_str::<client::ErrorResponse>(&res_body_string).map(|r| r.error))
                            .ok();

                        if let client::Retry::After(d) = dlg.http_failure(&res,
                                                              json_server_error,
                                                              server_error) {
                            sleep(d);
                            continue;
                        }
                        dlg.finished(false);
                        return match json::from_str::<client::ErrorResponse>(&res_body_string){
                            Err(_) => Err(client::Error::Failure(res)),
                            Ok(serr) => Err(client::Error::BadRequest(serr))
                        }
                    }
                    let result_value = {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        match json::from_str(&res_body_string) {
                            Ok(decoded) => (res, decoded),
                            Err(err) => {
                                dlg.response_json_decode_error(&res_body_string, &err);
                                return Err(client::Error::JsonDecodeError(res_body_string, err));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(result_value)
                }
            }
        }
    }


    /// The ID of the activity to get.
    ///
    /// Sets the *activity id* path property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn activity_id(mut self, new_value: &str) -> ActivityGetCall<'a> {
        self._activity_id = new_value.to_string();
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    /// 
    /// It should be used to handle progress information, and to implement a certain level of resilience.
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(mut self, new_value: &'a mut dyn client::Delegate) -> ActivityGetCall<'a> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *alt* (query-string) - Data format for the response.
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - An opaque string that represents a user for quota purposes. Must not exceed 40 characters.
    /// * *userIp* (query-string) - Deprecated. Please use quotaUser instead.
    pub fn param<T>(mut self, name: T, value: T) -> ActivityGetCall<'a>
                                                        where T: AsRef<str> {
        self._additional_params.insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead the default `Scope` variant
    /// `Scope::PluLogin`.
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    /// If `None` is specified, then all scopes will be removed and no default scope will be used either.
    /// In that case, you have to specify your API-key using the `key` parameter (see the `param()`
    /// function for details).
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<T, S>(mut self, scope: T) -> ActivityGetCall<'a>
                                                        where T: Into<Option<S>>,
                                                              S: AsRef<str> {
        match scope.into() {
          Some(scope) => self._scopes.insert(scope.as_ref().to_string(), ()),
          None => None,
        };
        self
    }
}


/// Shut down. See https://developers.google.com/+/api-shutdown for more details.
///
/// A builder for the *list* method supported by a *activity* resource.
/// It is not used directly, but through a `ActivityMethods` instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate yup_oauth2 as oauth2;
/// # extern crate google_plusdomains1 as plusdomains1;
/// # async fn dox() {
/// # use std::default::Default;
/// # use oauth2;
/// # use plusdomains1::PlusDomains;
/// 
/// # let secret: oauth2::ApplicationSecret = Default::default();
/// # let auth = yup_oauth2::InstalledFlowAuthenticator::builder(
/// #         secret,
/// #         yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     ).build().await.unwrap();
/// # let mut hub = PlusDomains::new(hyper::Client::builder().build(hyper_rustls::HttpsConnector::with_native_roots()), auth);
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.activities().list("userId", "collection")
///              .page_token("gubergren")
///              .max_results(26)
///              .doit().await;
/// # }
/// ```
pub struct ActivityListCall<'a>
    where  {

    hub: &'a PlusDomains<>,
    _user_id: String,
    _collection: String,
    _page_token: Option<String>,
    _max_results: Option<u32>,
    _delegate: Option<&'a mut dyn client::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeMap<String, ()>
}

impl<'a> client::CallBuilder for ActivityListCall<'a> {}

impl<'a> ActivityListCall<'a> {


    /// Perform the operation you have build so far.
    pub async fn doit(mut self) -> client::Result<(hyper::Response<hyper::body::Body>, ActivityFeed)> {
        use std::io::{Read, Seek};
        use hyper::header::{CONTENT_TYPE, CONTENT_LENGTH, AUTHORIZATION, USER_AGENT, LOCATION};
        use client::ToParts;
        let mut dd = client::DefaultDelegate;
        let mut dlg: &mut dyn client::Delegate = match self._delegate {
            Some(d) => d,
            None => &mut dd
        };
        dlg.begin(client::MethodInfo { id: "plusDomains.activities.list",
                               http_method: hyper::Method::GET });
        let mut params: Vec<(&str, String)> = Vec::with_capacity(6 + self._additional_params.len());
        params.push(("userId", self._user_id.to_string()));
        params.push(("collection", self._collection.to_string()));
        if let Some(value) = self._page_token {
            params.push(("pageToken", value.to_string()));
        }
        if let Some(value) = self._max_results {
            params.push(("maxResults", value.to_string()));
        }
        for &field in ["alt", "userId", "collection", "pageToken", "maxResults"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(client::Error::FieldClash(field));
            }
        }
        for (name, value) in self._additional_params.iter() {
            params.push((&name, value.clone()));
        }

        params.push(("alt", "json".to_string()));

        let mut url = self.hub._base_url.clone() + "people/{userId}/activities/{collection}";
        if self._scopes.len() == 0 {
            self._scopes.insert(Scope::PluLogin.as_ref().to_string(), ());
        }

        for &(find_this, param_name) in [("{userId}", "userId"), ("{collection}", "collection")].iter() {
            let mut replace_with: Option<&str> = None;
            for &(name, ref value) in params.iter() {
                if name == param_name {
                    replace_with = Some(value);
                    break;
                }
            }
            url = url.replace(find_this, replace_with.expect("to find substitution value in params"));
        }
        {
            let mut indices_for_removal: Vec<usize> = Vec::with_capacity(2);
            for param_name in ["collection", "userId"].iter() {
                if let Some(index) = params.iter().position(|t| &t.0 == param_name) {
                    indices_for_removal.push(index);
                }
            }
            for &index in indices_for_removal.iter() {
                params.remove(index);
            }
        }

        let url = url::Url::parse_with_params(&url, params).unwrap();



        loop {
            let token = match self.hub.auth.token(&self._scopes.keys().collect::<Vec<_>>()[..]).await {
                Ok(token) => token.clone(),
                Err(err) => {
                    match  dlg.token(&err) {
                        Some(token) => token,
                        None => {
                            dlg.finished(false);
                            return Err(client::Error::MissingToken(err))
                        }
                    }
                }
            };
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder().method(hyper::Method::GET).uri(url.clone().into_string())
                        .header(USER_AGENT, self.hub._user_agent.clone())                            .header(AUTHORIZATION, format!("Bearer {}", token.as_str()));


                        let request = req_builder
                        .body(hyper::body::Body::empty());

                client.request(request.unwrap()).await
                
            };

            match req_result {
                Err(err) => {
                    if let client::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d);
                        continue;
                    }
                    dlg.finished(false);
                    return Err(client::Error::HttpError(err))
                }
                Ok(mut res) => {
                    if !res.status().is_success() {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        let json_server_error = json::from_str::<client::JsonServerError>(&res_body_string).ok();
                        let server_error = json::from_str::<client::ServerError>(&res_body_string)
                            .or_else(|_| json::from_str::<client::ErrorResponse>(&res_body_string).map(|r| r.error))
                            .ok();

                        if let client::Retry::After(d) = dlg.http_failure(&res,
                                                              json_server_error,
                                                              server_error) {
                            sleep(d);
                            continue;
                        }
                        dlg.finished(false);
                        return match json::from_str::<client::ErrorResponse>(&res_body_string){
                            Err(_) => Err(client::Error::Failure(res)),
                            Ok(serr) => Err(client::Error::BadRequest(serr))
                        }
                    }
                    let result_value = {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        match json::from_str(&res_body_string) {
                            Ok(decoded) => (res, decoded),
                            Err(err) => {
                                dlg.response_json_decode_error(&res_body_string, &err);
                                return Err(client::Error::JsonDecodeError(res_body_string, err));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(result_value)
                }
            }
        }
    }


    /// The ID of the user to get activities for. The special value "me" can be used to indicate the authenticated user.
    ///
    /// Sets the *user id* path property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn user_id(mut self, new_value: &str) -> ActivityListCall<'a> {
        self._user_id = new_value.to_string();
        self
    }
    /// The collection of activities to list.
    ///
    /// Sets the *collection* path property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn collection(mut self, new_value: &str) -> ActivityListCall<'a> {
        self._collection = new_value.to_string();
        self
    }
    /// The continuation token, which is used to page through large result sets. To get the next page of results, set this parameter to the value of "nextPageToken" from the previous response.
    ///
    /// Sets the *page token* query property to the given value.
    pub fn page_token(mut self, new_value: &str) -> ActivityListCall<'a> {
        self._page_token = Some(new_value.to_string());
        self
    }
    /// The maximum number of activities to include in the response, which is used for paging. For any response, the actual number returned might be less than the specified maxResults.
    ///
    /// Sets the *max results* query property to the given value.
    pub fn max_results(mut self, new_value: u32) -> ActivityListCall<'a> {
        self._max_results = Some(new_value);
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    /// 
    /// It should be used to handle progress information, and to implement a certain level of resilience.
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(mut self, new_value: &'a mut dyn client::Delegate) -> ActivityListCall<'a> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *alt* (query-string) - Data format for the response.
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - An opaque string that represents a user for quota purposes. Must not exceed 40 characters.
    /// * *userIp* (query-string) - Deprecated. Please use quotaUser instead.
    pub fn param<T>(mut self, name: T, value: T) -> ActivityListCall<'a>
                                                        where T: AsRef<str> {
        self._additional_params.insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead the default `Scope` variant
    /// `Scope::PluLogin`.
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    /// If `None` is specified, then all scopes will be removed and no default scope will be used either.
    /// In that case, you have to specify your API-key using the `key` parameter (see the `param()`
    /// function for details).
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<T, S>(mut self, scope: T) -> ActivityListCall<'a>
                                                        where T: Into<Option<S>>,
                                                              S: AsRef<str> {
        match scope.into() {
          Some(scope) => self._scopes.insert(scope.as_ref().to_string(), ()),
          None => None,
        };
        self
    }
}


/// Shut down. See https://developers.google.com/+/api-shutdown for more details.
///
/// A builder for the *list* method supported by a *audience* resource.
/// It is not used directly, but through a `AudienceMethods` instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate yup_oauth2 as oauth2;
/// # extern crate google_plusdomains1 as plusdomains1;
/// # async fn dox() {
/// # use std::default::Default;
/// # use oauth2;
/// # use plusdomains1::PlusDomains;
/// 
/// # let secret: oauth2::ApplicationSecret = Default::default();
/// # let auth = yup_oauth2::InstalledFlowAuthenticator::builder(
/// #         secret,
/// #         yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     ).build().await.unwrap();
/// # let mut hub = PlusDomains::new(hyper::Client::builder().build(hyper_rustls::HttpsConnector::with_native_roots()), auth);
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.audiences().list("userId")
///              .page_token("ea")
///              .max_results(46)
///              .doit().await;
/// # }
/// ```
pub struct AudienceListCall<'a>
    where  {

    hub: &'a PlusDomains<>,
    _user_id: String,
    _page_token: Option<String>,
    _max_results: Option<u32>,
    _delegate: Option<&'a mut dyn client::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeMap<String, ()>
}

impl<'a> client::CallBuilder for AudienceListCall<'a> {}

impl<'a> AudienceListCall<'a> {


    /// Perform the operation you have build so far.
    pub async fn doit(mut self) -> client::Result<(hyper::Response<hyper::body::Body>, AudiencesFeed)> {
        use std::io::{Read, Seek};
        use hyper::header::{CONTENT_TYPE, CONTENT_LENGTH, AUTHORIZATION, USER_AGENT, LOCATION};
        use client::ToParts;
        let mut dd = client::DefaultDelegate;
        let mut dlg: &mut dyn client::Delegate = match self._delegate {
            Some(d) => d,
            None => &mut dd
        };
        dlg.begin(client::MethodInfo { id: "plusDomains.audiences.list",
                               http_method: hyper::Method::GET });
        let mut params: Vec<(&str, String)> = Vec::with_capacity(5 + self._additional_params.len());
        params.push(("userId", self._user_id.to_string()));
        if let Some(value) = self._page_token {
            params.push(("pageToken", value.to_string()));
        }
        if let Some(value) = self._max_results {
            params.push(("maxResults", value.to_string()));
        }
        for &field in ["alt", "userId", "pageToken", "maxResults"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(client::Error::FieldClash(field));
            }
        }
        for (name, value) in self._additional_params.iter() {
            params.push((&name, value.clone()));
        }

        params.push(("alt", "json".to_string()));

        let mut url = self.hub._base_url.clone() + "people/{userId}/audiences";
        if self._scopes.len() == 0 {
            self._scopes.insert(Scope::PluCircleRead.as_ref().to_string(), ());
        }

        for &(find_this, param_name) in [("{userId}", "userId")].iter() {
            let mut replace_with: Option<&str> = None;
            for &(name, ref value) in params.iter() {
                if name == param_name {
                    replace_with = Some(value);
                    break;
                }
            }
            url = url.replace(find_this, replace_with.expect("to find substitution value in params"));
        }
        {
            let mut indices_for_removal: Vec<usize> = Vec::with_capacity(1);
            for param_name in ["userId"].iter() {
                if let Some(index) = params.iter().position(|t| &t.0 == param_name) {
                    indices_for_removal.push(index);
                }
            }
            for &index in indices_for_removal.iter() {
                params.remove(index);
            }
        }

        let url = url::Url::parse_with_params(&url, params).unwrap();



        loop {
            let token = match self.hub.auth.token(&self._scopes.keys().collect::<Vec<_>>()[..]).await {
                Ok(token) => token.clone(),
                Err(err) => {
                    match  dlg.token(&err) {
                        Some(token) => token,
                        None => {
                            dlg.finished(false);
                            return Err(client::Error::MissingToken(err))
                        }
                    }
                }
            };
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder().method(hyper::Method::GET).uri(url.clone().into_string())
                        .header(USER_AGENT, self.hub._user_agent.clone())                            .header(AUTHORIZATION, format!("Bearer {}", token.as_str()));


                        let request = req_builder
                        .body(hyper::body::Body::empty());

                client.request(request.unwrap()).await
                
            };

            match req_result {
                Err(err) => {
                    if let client::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d);
                        continue;
                    }
                    dlg.finished(false);
                    return Err(client::Error::HttpError(err))
                }
                Ok(mut res) => {
                    if !res.status().is_success() {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        let json_server_error = json::from_str::<client::JsonServerError>(&res_body_string).ok();
                        let server_error = json::from_str::<client::ServerError>(&res_body_string)
                            .or_else(|_| json::from_str::<client::ErrorResponse>(&res_body_string).map(|r| r.error))
                            .ok();

                        if let client::Retry::After(d) = dlg.http_failure(&res,
                                                              json_server_error,
                                                              server_error) {
                            sleep(d);
                            continue;
                        }
                        dlg.finished(false);
                        return match json::from_str::<client::ErrorResponse>(&res_body_string){
                            Err(_) => Err(client::Error::Failure(res)),
                            Ok(serr) => Err(client::Error::BadRequest(serr))
                        }
                    }
                    let result_value = {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        match json::from_str(&res_body_string) {
                            Ok(decoded) => (res, decoded),
                            Err(err) => {
                                dlg.response_json_decode_error(&res_body_string, &err);
                                return Err(client::Error::JsonDecodeError(res_body_string, err));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(result_value)
                }
            }
        }
    }


    /// The ID of the user to get audiences for. The special value "me" can be used to indicate the authenticated user.
    ///
    /// Sets the *user id* path property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn user_id(mut self, new_value: &str) -> AudienceListCall<'a> {
        self._user_id = new_value.to_string();
        self
    }
    /// The continuation token, which is used to page through large result sets. To get the next page of results, set this parameter to the value of "nextPageToken" from the previous response.
    ///
    /// Sets the *page token* query property to the given value.
    pub fn page_token(mut self, new_value: &str) -> AudienceListCall<'a> {
        self._page_token = Some(new_value.to_string());
        self
    }
    /// The maximum number of circles to include in the response, which is used for paging. For any response, the actual number returned might be less than the specified maxResults.
    ///
    /// Sets the *max results* query property to the given value.
    pub fn max_results(mut self, new_value: u32) -> AudienceListCall<'a> {
        self._max_results = Some(new_value);
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    /// 
    /// It should be used to handle progress information, and to implement a certain level of resilience.
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(mut self, new_value: &'a mut dyn client::Delegate) -> AudienceListCall<'a> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *alt* (query-string) - Data format for the response.
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - An opaque string that represents a user for quota purposes. Must not exceed 40 characters.
    /// * *userIp* (query-string) - Deprecated. Please use quotaUser instead.
    pub fn param<T>(mut self, name: T, value: T) -> AudienceListCall<'a>
                                                        where T: AsRef<str> {
        self._additional_params.insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead the default `Scope` variant
    /// `Scope::PluCircleRead`.
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    /// If `None` is specified, then all scopes will be removed and no default scope will be used either.
    /// In that case, you have to specify your API-key using the `key` parameter (see the `param()`
    /// function for details).
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<T, S>(mut self, scope: T) -> AudienceListCall<'a>
                                                        where T: Into<Option<S>>,
                                                              S: AsRef<str> {
        match scope.into() {
          Some(scope) => self._scopes.insert(scope.as_ref().to_string(), ()),
          None => None,
        };
        self
    }
}


/// Shut down. See https://developers.google.com/+/api-shutdown for more details.
///
/// A builder for the *list* method supported by a *circle* resource.
/// It is not used directly, but through a `CircleMethods` instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate yup_oauth2 as oauth2;
/// # extern crate google_plusdomains1 as plusdomains1;
/// # async fn dox() {
/// # use std::default::Default;
/// # use oauth2;
/// # use plusdomains1::PlusDomains;
/// 
/// # let secret: oauth2::ApplicationSecret = Default::default();
/// # let auth = yup_oauth2::InstalledFlowAuthenticator::builder(
/// #         secret,
/// #         yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     ).build().await.unwrap();
/// # let mut hub = PlusDomains::new(hyper::Client::builder().build(hyper_rustls::HttpsConnector::with_native_roots()), auth);
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.circles().list("userId")
///              .page_token("amet")
///              .max_results(81)
///              .doit().await;
/// # }
/// ```
pub struct CircleListCall<'a>
    where  {

    hub: &'a PlusDomains<>,
    _user_id: String,
    _page_token: Option<String>,
    _max_results: Option<u32>,
    _delegate: Option<&'a mut dyn client::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeMap<String, ()>
}

impl<'a> client::CallBuilder for CircleListCall<'a> {}

impl<'a> CircleListCall<'a> {


    /// Perform the operation you have build so far.
    pub async fn doit(mut self) -> client::Result<(hyper::Response<hyper::body::Body>, CircleFeed)> {
        use std::io::{Read, Seek};
        use hyper::header::{CONTENT_TYPE, CONTENT_LENGTH, AUTHORIZATION, USER_AGENT, LOCATION};
        use client::ToParts;
        let mut dd = client::DefaultDelegate;
        let mut dlg: &mut dyn client::Delegate = match self._delegate {
            Some(d) => d,
            None => &mut dd
        };
        dlg.begin(client::MethodInfo { id: "plusDomains.circles.list",
                               http_method: hyper::Method::GET });
        let mut params: Vec<(&str, String)> = Vec::with_capacity(5 + self._additional_params.len());
        params.push(("userId", self._user_id.to_string()));
        if let Some(value) = self._page_token {
            params.push(("pageToken", value.to_string()));
        }
        if let Some(value) = self._max_results {
            params.push(("maxResults", value.to_string()));
        }
        for &field in ["alt", "userId", "pageToken", "maxResults"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(client::Error::FieldClash(field));
            }
        }
        for (name, value) in self._additional_params.iter() {
            params.push((&name, value.clone()));
        }

        params.push(("alt", "json".to_string()));

        let mut url = self.hub._base_url.clone() + "people/{userId}/circles";
        if self._scopes.len() == 0 {
            self._scopes.insert(Scope::PluCircleRead.as_ref().to_string(), ());
        }

        for &(find_this, param_name) in [("{userId}", "userId")].iter() {
            let mut replace_with: Option<&str> = None;
            for &(name, ref value) in params.iter() {
                if name == param_name {
                    replace_with = Some(value);
                    break;
                }
            }
            url = url.replace(find_this, replace_with.expect("to find substitution value in params"));
        }
        {
            let mut indices_for_removal: Vec<usize> = Vec::with_capacity(1);
            for param_name in ["userId"].iter() {
                if let Some(index) = params.iter().position(|t| &t.0 == param_name) {
                    indices_for_removal.push(index);
                }
            }
            for &index in indices_for_removal.iter() {
                params.remove(index);
            }
        }

        let url = url::Url::parse_with_params(&url, params).unwrap();



        loop {
            let token = match self.hub.auth.token(&self._scopes.keys().collect::<Vec<_>>()[..]).await {
                Ok(token) => token.clone(),
                Err(err) => {
                    match  dlg.token(&err) {
                        Some(token) => token,
                        None => {
                            dlg.finished(false);
                            return Err(client::Error::MissingToken(err))
                        }
                    }
                }
            };
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder().method(hyper::Method::GET).uri(url.clone().into_string())
                        .header(USER_AGENT, self.hub._user_agent.clone())                            .header(AUTHORIZATION, format!("Bearer {}", token.as_str()));


                        let request = req_builder
                        .body(hyper::body::Body::empty());

                client.request(request.unwrap()).await
                
            };

            match req_result {
                Err(err) => {
                    if let client::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d);
                        continue;
                    }
                    dlg.finished(false);
                    return Err(client::Error::HttpError(err))
                }
                Ok(mut res) => {
                    if !res.status().is_success() {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        let json_server_error = json::from_str::<client::JsonServerError>(&res_body_string).ok();
                        let server_error = json::from_str::<client::ServerError>(&res_body_string)
                            .or_else(|_| json::from_str::<client::ErrorResponse>(&res_body_string).map(|r| r.error))
                            .ok();

                        if let client::Retry::After(d) = dlg.http_failure(&res,
                                                              json_server_error,
                                                              server_error) {
                            sleep(d);
                            continue;
                        }
                        dlg.finished(false);
                        return match json::from_str::<client::ErrorResponse>(&res_body_string){
                            Err(_) => Err(client::Error::Failure(res)),
                            Ok(serr) => Err(client::Error::BadRequest(serr))
                        }
                    }
                    let result_value = {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        match json::from_str(&res_body_string) {
                            Ok(decoded) => (res, decoded),
                            Err(err) => {
                                dlg.response_json_decode_error(&res_body_string, &err);
                                return Err(client::Error::JsonDecodeError(res_body_string, err));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(result_value)
                }
            }
        }
    }


    /// The ID of the user to get circles for. The special value "me" can be used to indicate the authenticated user.
    ///
    /// Sets the *user id* path property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn user_id(mut self, new_value: &str) -> CircleListCall<'a> {
        self._user_id = new_value.to_string();
        self
    }
    /// The continuation token, which is used to page through large result sets. To get the next page of results, set this parameter to the value of "nextPageToken" from the previous response.
    ///
    /// Sets the *page token* query property to the given value.
    pub fn page_token(mut self, new_value: &str) -> CircleListCall<'a> {
        self._page_token = Some(new_value.to_string());
        self
    }
    /// The maximum number of circles to include in the response, which is used for paging. For any response, the actual number returned might be less than the specified maxResults.
    ///
    /// Sets the *max results* query property to the given value.
    pub fn max_results(mut self, new_value: u32) -> CircleListCall<'a> {
        self._max_results = Some(new_value);
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    /// 
    /// It should be used to handle progress information, and to implement a certain level of resilience.
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(mut self, new_value: &'a mut dyn client::Delegate) -> CircleListCall<'a> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *alt* (query-string) - Data format for the response.
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - An opaque string that represents a user for quota purposes. Must not exceed 40 characters.
    /// * *userIp* (query-string) - Deprecated. Please use quotaUser instead.
    pub fn param<T>(mut self, name: T, value: T) -> CircleListCall<'a>
                                                        where T: AsRef<str> {
        self._additional_params.insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead the default `Scope` variant
    /// `Scope::PluCircleRead`.
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    /// If `None` is specified, then all scopes will be removed and no default scope will be used either.
    /// In that case, you have to specify your API-key using the `key` parameter (see the `param()`
    /// function for details).
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<T, S>(mut self, scope: T) -> CircleListCall<'a>
                                                        where T: Into<Option<S>>,
                                                              S: AsRef<str> {
        match scope.into() {
          Some(scope) => self._scopes.insert(scope.as_ref().to_string(), ()),
          None => None,
        };
        self
    }
}


/// Shut down. See https://developers.google.com/+/api-shutdown for more details.
///
/// A builder for the *get* method supported by a *comment* resource.
/// It is not used directly, but through a `CommentMethods` instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate yup_oauth2 as oauth2;
/// # extern crate google_plusdomains1 as plusdomains1;
/// # async fn dox() {
/// # use std::default::Default;
/// # use oauth2;
/// # use plusdomains1::PlusDomains;
/// 
/// # let secret: oauth2::ApplicationSecret = Default::default();
/// # let auth = yup_oauth2::InstalledFlowAuthenticator::builder(
/// #         secret,
/// #         yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     ).build().await.unwrap();
/// # let mut hub = PlusDomains::new(hyper::Client::builder().build(hyper_rustls::HttpsConnector::with_native_roots()), auth);
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.comments().get("commentId")
///              .doit().await;
/// # }
/// ```
pub struct CommentGetCall<'a>
    where  {

    hub: &'a PlusDomains<>,
    _comment_id: String,
    _delegate: Option<&'a mut dyn client::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeMap<String, ()>
}

impl<'a> client::CallBuilder for CommentGetCall<'a> {}

impl<'a> CommentGetCall<'a> {


    /// Perform the operation you have build so far.
    pub async fn doit(mut self) -> client::Result<(hyper::Response<hyper::body::Body>, Comment)> {
        use std::io::{Read, Seek};
        use hyper::header::{CONTENT_TYPE, CONTENT_LENGTH, AUTHORIZATION, USER_AGENT, LOCATION};
        use client::ToParts;
        let mut dd = client::DefaultDelegate;
        let mut dlg: &mut dyn client::Delegate = match self._delegate {
            Some(d) => d,
            None => &mut dd
        };
        dlg.begin(client::MethodInfo { id: "plusDomains.comments.get",
                               http_method: hyper::Method::GET });
        let mut params: Vec<(&str, String)> = Vec::with_capacity(3 + self._additional_params.len());
        params.push(("commentId", self._comment_id.to_string()));
        for &field in ["alt", "commentId"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(client::Error::FieldClash(field));
            }
        }
        for (name, value) in self._additional_params.iter() {
            params.push((&name, value.clone()));
        }

        params.push(("alt", "json".to_string()));

        let mut url = self.hub._base_url.clone() + "comments/{commentId}";
        if self._scopes.len() == 0 {
            self._scopes.insert(Scope::PluLogin.as_ref().to_string(), ());
        }

        for &(find_this, param_name) in [("{commentId}", "commentId")].iter() {
            let mut replace_with: Option<&str> = None;
            for &(name, ref value) in params.iter() {
                if name == param_name {
                    replace_with = Some(value);
                    break;
                }
            }
            url = url.replace(find_this, replace_with.expect("to find substitution value in params"));
        }
        {
            let mut indices_for_removal: Vec<usize> = Vec::with_capacity(1);
            for param_name in ["commentId"].iter() {
                if let Some(index) = params.iter().position(|t| &t.0 == param_name) {
                    indices_for_removal.push(index);
                }
            }
            for &index in indices_for_removal.iter() {
                params.remove(index);
            }
        }

        let url = url::Url::parse_with_params(&url, params).unwrap();



        loop {
            let token = match self.hub.auth.token(&self._scopes.keys().collect::<Vec<_>>()[..]).await {
                Ok(token) => token.clone(),
                Err(err) => {
                    match  dlg.token(&err) {
                        Some(token) => token,
                        None => {
                            dlg.finished(false);
                            return Err(client::Error::MissingToken(err))
                        }
                    }
                }
            };
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder().method(hyper::Method::GET).uri(url.clone().into_string())
                        .header(USER_AGENT, self.hub._user_agent.clone())                            .header(AUTHORIZATION, format!("Bearer {}", token.as_str()));


                        let request = req_builder
                        .body(hyper::body::Body::empty());

                client.request(request.unwrap()).await
                
            };

            match req_result {
                Err(err) => {
                    if let client::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d);
                        continue;
                    }
                    dlg.finished(false);
                    return Err(client::Error::HttpError(err))
                }
                Ok(mut res) => {
                    if !res.status().is_success() {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        let json_server_error = json::from_str::<client::JsonServerError>(&res_body_string).ok();
                        let server_error = json::from_str::<client::ServerError>(&res_body_string)
                            .or_else(|_| json::from_str::<client::ErrorResponse>(&res_body_string).map(|r| r.error))
                            .ok();

                        if let client::Retry::After(d) = dlg.http_failure(&res,
                                                              json_server_error,
                                                              server_error) {
                            sleep(d);
                            continue;
                        }
                        dlg.finished(false);
                        return match json::from_str::<client::ErrorResponse>(&res_body_string){
                            Err(_) => Err(client::Error::Failure(res)),
                            Ok(serr) => Err(client::Error::BadRequest(serr))
                        }
                    }
                    let result_value = {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        match json::from_str(&res_body_string) {
                            Ok(decoded) => (res, decoded),
                            Err(err) => {
                                dlg.response_json_decode_error(&res_body_string, &err);
                                return Err(client::Error::JsonDecodeError(res_body_string, err));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(result_value)
                }
            }
        }
    }


    /// The ID of the comment to get.
    ///
    /// Sets the *comment id* path property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn comment_id(mut self, new_value: &str) -> CommentGetCall<'a> {
        self._comment_id = new_value.to_string();
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    /// 
    /// It should be used to handle progress information, and to implement a certain level of resilience.
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(mut self, new_value: &'a mut dyn client::Delegate) -> CommentGetCall<'a> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *alt* (query-string) - Data format for the response.
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - An opaque string that represents a user for quota purposes. Must not exceed 40 characters.
    /// * *userIp* (query-string) - Deprecated. Please use quotaUser instead.
    pub fn param<T>(mut self, name: T, value: T) -> CommentGetCall<'a>
                                                        where T: AsRef<str> {
        self._additional_params.insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead the default `Scope` variant
    /// `Scope::PluLogin`.
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    /// If `None` is specified, then all scopes will be removed and no default scope will be used either.
    /// In that case, you have to specify your API-key using the `key` parameter (see the `param()`
    /// function for details).
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<T, S>(mut self, scope: T) -> CommentGetCall<'a>
                                                        where T: Into<Option<S>>,
                                                              S: AsRef<str> {
        match scope.into() {
          Some(scope) => self._scopes.insert(scope.as_ref().to_string(), ()),
          None => None,
        };
        self
    }
}


/// Shut down. See https://developers.google.com/+/api-shutdown for more details.
///
/// A builder for the *list* method supported by a *comment* resource.
/// It is not used directly, but through a `CommentMethods` instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate yup_oauth2 as oauth2;
/// # extern crate google_plusdomains1 as plusdomains1;
/// # async fn dox() {
/// # use std::default::Default;
/// # use oauth2;
/// # use plusdomains1::PlusDomains;
/// 
/// # let secret: oauth2::ApplicationSecret = Default::default();
/// # let auth = yup_oauth2::InstalledFlowAuthenticator::builder(
/// #         secret,
/// #         yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     ).build().await.unwrap();
/// # let mut hub = PlusDomains::new(hyper::Client::builder().build(hyper_rustls::HttpsConnector::with_native_roots()), auth);
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.comments().list("activityId")
///              .sort_order("ut")
///              .page_token("gubergren")
///              .max_results(85)
///              .doit().await;
/// # }
/// ```
pub struct CommentListCall<'a>
    where  {

    hub: &'a PlusDomains<>,
    _activity_id: String,
    _sort_order: Option<String>,
    _page_token: Option<String>,
    _max_results: Option<u32>,
    _delegate: Option<&'a mut dyn client::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeMap<String, ()>
}

impl<'a> client::CallBuilder for CommentListCall<'a> {}

impl<'a> CommentListCall<'a> {


    /// Perform the operation you have build so far.
    pub async fn doit(mut self) -> client::Result<(hyper::Response<hyper::body::Body>, CommentFeed)> {
        use std::io::{Read, Seek};
        use hyper::header::{CONTENT_TYPE, CONTENT_LENGTH, AUTHORIZATION, USER_AGENT, LOCATION};
        use client::ToParts;
        let mut dd = client::DefaultDelegate;
        let mut dlg: &mut dyn client::Delegate = match self._delegate {
            Some(d) => d,
            None => &mut dd
        };
        dlg.begin(client::MethodInfo { id: "plusDomains.comments.list",
                               http_method: hyper::Method::GET });
        let mut params: Vec<(&str, String)> = Vec::with_capacity(6 + self._additional_params.len());
        params.push(("activityId", self._activity_id.to_string()));
        if let Some(value) = self._sort_order {
            params.push(("sortOrder", value.to_string()));
        }
        if let Some(value) = self._page_token {
            params.push(("pageToken", value.to_string()));
        }
        if let Some(value) = self._max_results {
            params.push(("maxResults", value.to_string()));
        }
        for &field in ["alt", "activityId", "sortOrder", "pageToken", "maxResults"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(client::Error::FieldClash(field));
            }
        }
        for (name, value) in self._additional_params.iter() {
            params.push((&name, value.clone()));
        }

        params.push(("alt", "json".to_string()));

        let mut url = self.hub._base_url.clone() + "activities/{activityId}/comments";
        if self._scopes.len() == 0 {
            self._scopes.insert(Scope::PluLogin.as_ref().to_string(), ());
        }

        for &(find_this, param_name) in [("{activityId}", "activityId")].iter() {
            let mut replace_with: Option<&str> = None;
            for &(name, ref value) in params.iter() {
                if name == param_name {
                    replace_with = Some(value);
                    break;
                }
            }
            url = url.replace(find_this, replace_with.expect("to find substitution value in params"));
        }
        {
            let mut indices_for_removal: Vec<usize> = Vec::with_capacity(1);
            for param_name in ["activityId"].iter() {
                if let Some(index) = params.iter().position(|t| &t.0 == param_name) {
                    indices_for_removal.push(index);
                }
            }
            for &index in indices_for_removal.iter() {
                params.remove(index);
            }
        }

        let url = url::Url::parse_with_params(&url, params).unwrap();



        loop {
            let token = match self.hub.auth.token(&self._scopes.keys().collect::<Vec<_>>()[..]).await {
                Ok(token) => token.clone(),
                Err(err) => {
                    match  dlg.token(&err) {
                        Some(token) => token,
                        None => {
                            dlg.finished(false);
                            return Err(client::Error::MissingToken(err))
                        }
                    }
                }
            };
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder().method(hyper::Method::GET).uri(url.clone().into_string())
                        .header(USER_AGENT, self.hub._user_agent.clone())                            .header(AUTHORIZATION, format!("Bearer {}", token.as_str()));


                        let request = req_builder
                        .body(hyper::body::Body::empty());

                client.request(request.unwrap()).await
                
            };

            match req_result {
                Err(err) => {
                    if let client::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d);
                        continue;
                    }
                    dlg.finished(false);
                    return Err(client::Error::HttpError(err))
                }
                Ok(mut res) => {
                    if !res.status().is_success() {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        let json_server_error = json::from_str::<client::JsonServerError>(&res_body_string).ok();
                        let server_error = json::from_str::<client::ServerError>(&res_body_string)
                            .or_else(|_| json::from_str::<client::ErrorResponse>(&res_body_string).map(|r| r.error))
                            .ok();

                        if let client::Retry::After(d) = dlg.http_failure(&res,
                                                              json_server_error,
                                                              server_error) {
                            sleep(d);
                            continue;
                        }
                        dlg.finished(false);
                        return match json::from_str::<client::ErrorResponse>(&res_body_string){
                            Err(_) => Err(client::Error::Failure(res)),
                            Ok(serr) => Err(client::Error::BadRequest(serr))
                        }
                    }
                    let result_value = {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        match json::from_str(&res_body_string) {
                            Ok(decoded) => (res, decoded),
                            Err(err) => {
                                dlg.response_json_decode_error(&res_body_string, &err);
                                return Err(client::Error::JsonDecodeError(res_body_string, err));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(result_value)
                }
            }
        }
    }


    /// The ID of the activity to get comments for.
    ///
    /// Sets the *activity id* path property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn activity_id(mut self, new_value: &str) -> CommentListCall<'a> {
        self._activity_id = new_value.to_string();
        self
    }
    /// The order in which to sort the list of comments.
    ///
    /// Sets the *sort order* query property to the given value.
    pub fn sort_order(mut self, new_value: &str) -> CommentListCall<'a> {
        self._sort_order = Some(new_value.to_string());
        self
    }
    /// The continuation token, which is used to page through large result sets. To get the next page of results, set this parameter to the value of "nextPageToken" from the previous response.
    ///
    /// Sets the *page token* query property to the given value.
    pub fn page_token(mut self, new_value: &str) -> CommentListCall<'a> {
        self._page_token = Some(new_value.to_string());
        self
    }
    /// The maximum number of comments to include in the response, which is used for paging. For any response, the actual number returned might be less than the specified maxResults.
    ///
    /// Sets the *max results* query property to the given value.
    pub fn max_results(mut self, new_value: u32) -> CommentListCall<'a> {
        self._max_results = Some(new_value);
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    /// 
    /// It should be used to handle progress information, and to implement a certain level of resilience.
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(mut self, new_value: &'a mut dyn client::Delegate) -> CommentListCall<'a> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *alt* (query-string) - Data format for the response.
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - An opaque string that represents a user for quota purposes. Must not exceed 40 characters.
    /// * *userIp* (query-string) - Deprecated. Please use quotaUser instead.
    pub fn param<T>(mut self, name: T, value: T) -> CommentListCall<'a>
                                                        where T: AsRef<str> {
        self._additional_params.insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead the default `Scope` variant
    /// `Scope::PluLogin`.
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    /// If `None` is specified, then all scopes will be removed and no default scope will be used either.
    /// In that case, you have to specify your API-key using the `key` parameter (see the `param()`
    /// function for details).
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<T, S>(mut self, scope: T) -> CommentListCall<'a>
                                                        where T: Into<Option<S>>,
                                                              S: AsRef<str> {
        match scope.into() {
          Some(scope) => self._scopes.insert(scope.as_ref().to_string(), ()),
          None => None,
        };
        self
    }
}


/// Shut down. See https://developers.google.com/+/api-shutdown for more details.
///
/// A builder for the *insert* method supported by a *media* resource.
/// It is not used directly, but through a `MediaMethods` instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate yup_oauth2 as oauth2;
/// # extern crate google_plusdomains1 as plusdomains1;
/// use plusdomains1::api::Media;
/// use std::fs;
/// # async fn dox() {
/// # use std::default::Default;
/// # use oauth2;
/// # use plusdomains1::PlusDomains;
/// 
/// # let secret: oauth2::ApplicationSecret = Default::default();
/// # let auth = yup_oauth2::InstalledFlowAuthenticator::builder(
/// #         secret,
/// #         yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     ).build().await.unwrap();
/// # let mut hub = PlusDomains::new(hyper::Client::builder().build(hyper_rustls::HttpsConnector::with_native_roots()), auth);
/// // As the method needs a request, you would usually fill it with the desired information
/// // into the respective structure. Some of the parts shown here might not be applicable !
/// // Values shown here are possibly random and not representative !
/// let mut req = Media::default();
/// 
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `upload_resumable(...)`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.media().insert(req, "userId", "collection")
///              .upload_resumable(fs::File::open("file.ext").unwrap(), "application/octet-stream".parse().unwrap()).await;
/// # }
/// ```
pub struct MediaInsertCall<'a>
    where  {

    hub: &'a PlusDomains<>,
    _request: Media,
    _user_id: String,
    _collection: String,
    _delegate: Option<&'a mut dyn client::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeMap<String, ()>
}

impl<'a> client::CallBuilder for MediaInsertCall<'a> {}

impl<'a> MediaInsertCall<'a> {


    /// Perform the operation you have build so far.
    async fn doit<RS>(mut self, mut reader: RS, reader_mime_type: mime::Mime, protocol: &'static str) -> client::Result<(hyper::Response<hyper::body::Body>, Media)>
		where RS: client::ReadSeek {
        use std::io::{Read, Seek};
        use hyper::header::{CONTENT_TYPE, CONTENT_LENGTH, AUTHORIZATION, USER_AGENT, LOCATION};
        use client::ToParts;
        let mut dd = client::DefaultDelegate;
        let mut dlg: &mut dyn client::Delegate = match self._delegate {
            Some(d) => d,
            None => &mut dd
        };
        dlg.begin(client::MethodInfo { id: "plusDomains.media.insert",
                               http_method: hyper::Method::POST });
        let mut params: Vec<(&str, String)> = Vec::with_capacity(5 + self._additional_params.len());
        params.push(("userId", self._user_id.to_string()));
        params.push(("collection", self._collection.to_string()));
        for &field in ["alt", "userId", "collection"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(client::Error::FieldClash(field));
            }
        }
        for (name, value) in self._additional_params.iter() {
            params.push((&name, value.clone()));
        }

        params.push(("alt", "json".to_string()));

        let (mut url, upload_type) =
            if protocol == "resumable" {
                (self.hub._root_url.clone() + "resumable/upload/plusDomains/v1/people/{userId}/media/{collection}", "resumable")
            } else if protocol == "simple" {
                (self.hub._root_url.clone() + "upload/plusDomains/v1/people/{userId}/media/{collection}", "multipart")
            } else {
                unreachable!()
            };
        params.push(("uploadType", upload_type.to_string()));
        if self._scopes.len() == 0 {
            self._scopes.insert(Scope::PluLogin.as_ref().to_string(), ());
        }

        for &(find_this, param_name) in [("{userId}", "userId"), ("{collection}", "collection")].iter() {
            let mut replace_with: Option<&str> = None;
            for &(name, ref value) in params.iter() {
                if name == param_name {
                    replace_with = Some(value);
                    break;
                }
            }
            url = url.replace(find_this, replace_with.expect("to find substitution value in params"));
        }
        {
            let mut indices_for_removal: Vec<usize> = Vec::with_capacity(2);
            for param_name in ["collection", "userId"].iter() {
                if let Some(index) = params.iter().position(|t| &t.0 == param_name) {
                    indices_for_removal.push(index);
                }
            }
            for &index in indices_for_removal.iter() {
                params.remove(index);
            }
        }

        let url = url::Url::parse_with_params(&url, params).unwrap();

        let mut json_mime_type: mime::Mime = "application/json".parse().unwrap();
        let mut request_value_reader =
            {
                let mut value = json::value::to_value(&self._request).expect("serde to work");
                client::remove_json_null_values(&mut value);
                let mut dst = io::Cursor::new(Vec::with_capacity(128));
                json::to_writer(&mut dst, &value).unwrap();
                dst
            };
        let request_size = request_value_reader.seek(io::SeekFrom::End(0)).unwrap();
        request_value_reader.seek(io::SeekFrom::Start(0)).unwrap();

        let mut should_ask_dlg_for_url = false;
        let mut upload_url_from_server;
        let mut upload_url: Option<String> = None;

        loop {
            let token = match self.hub.auth.token(&self._scopes.keys().collect::<Vec<_>>()[..]).await {
                Ok(token) => token.clone(),
                Err(err) => {
                    match  dlg.token(&err) {
                        Some(token) => token,
                        None => {
                            dlg.finished(false);
                            return Err(client::Error::MissingToken(err))
                        }
                    }
                }
            };
            request_value_reader.seek(io::SeekFrom::Start(0)).unwrap();
            let mut req_result = {
                if should_ask_dlg_for_url && (upload_url = dlg.upload_url()) == () && upload_url.is_some() {
                    should_ask_dlg_for_url = false;
                    upload_url_from_server = false;
                    let url = upload_url.as_ref().and_then(|s| Some(url::Url::parse(s).unwrap())).unwrap();
                    Ok(hyper::Response::builder()
                        .status(hyper::StatusCode::OK)
                        .header("Localtion", upload_url.as_ref().unwrap().clone())
                        .body(hyper::body::Body::empty())
                        .unwrap())
                } else {
                    let mut mp_reader: client::MultiPartReader = Default::default();
                    let (mut body_reader, content_type) = match protocol {
                        "simple" => {
                            mp_reader.reserve_exact(2);
                            let size = reader.seek(io::SeekFrom::End(0)).unwrap();
                        reader.seek(io::SeekFrom::Start(0)).unwrap();
                        
                            mp_reader.add_part(&mut request_value_reader, request_size, json_mime_type.clone())
                                     .add_part(&mut reader, size, reader_mime_type.clone());
                            let mime_type = mp_reader.mime_type();
                            (&mut mp_reader as &mut dyn io::Read, (CONTENT_TYPE, mime_type.to_string()))
                        },
                        _ => (&mut request_value_reader as &mut dyn io::Read, (CONTENT_TYPE, json_mime_type.to_string())),
                    };
                    let client = &self.hub.client;
                    dlg.pre_request();
                    let mut req_builder = hyper::Request::builder().method(hyper::Method::POST).uri(url.clone().into_string())
                            .header(USER_AGENT, self.hub._user_agent.clone())                            .header(AUTHORIZATION, format!("Bearer {}", token.as_str()));
    
                    upload_url_from_server = true;
                    if protocol == "resumable" {
                        req_builder = req_builder.header("X-Upload-Content-Type", format!("{}", reader_mime_type));
                    }
    
                            let mut body_reader_bytes = vec![];
                            body_reader.read_to_end(&mut body_reader_bytes).unwrap();
                            let request = req_builder
                            .header(content_type.0, content_type.1.to_string())
                            .body(hyper::body::Body::from(body_reader_bytes));
    
                    client.request(request.unwrap()).await
                    
                }
            };

            match req_result {
                Err(err) => {
                    if let client::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d);
                        continue;
                    }
                    dlg.finished(false);
                    return Err(client::Error::HttpError(err))
                }
                Ok(mut res) => {
                    if !res.status().is_success() {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        let json_server_error = json::from_str::<client::JsonServerError>(&res_body_string).ok();
                        let server_error = json::from_str::<client::ServerError>(&res_body_string)
                            .or_else(|_| json::from_str::<client::ErrorResponse>(&res_body_string).map(|r| r.error))
                            .ok();

                        if let client::Retry::After(d) = dlg.http_failure(&res,
                                                              json_server_error,
                                                              server_error) {
                            sleep(d);
                            continue;
                        }
                        dlg.finished(false);
                        return match json::from_str::<client::ErrorResponse>(&res_body_string){
                            Err(_) => Err(client::Error::Failure(res)),
                            Ok(serr) => Err(client::Error::BadRequest(serr))
                        }
                    }
                    if protocol == "resumable" {
                        let size = reader.seek(io::SeekFrom::End(0)).unwrap();
                        reader.seek(io::SeekFrom::Start(0)).unwrap();
                        
                        let upload_result = {
                            let url_str = &res.headers().get("Location").expect("LOCATION header is part of protocol").to_str().unwrap();
                            if upload_url_from_server {
                                dlg.store_upload_url(Some(url_str));
                            }

                            client::ResumableUploadHelper {
                                client: &self.hub.client,
                                delegate: dlg,
                                start_at: if upload_url_from_server { Some(0) } else { None },
                                auth: &self.hub.auth,
                                user_agent: &self.hub._user_agent,
                                auth_header: format!("Bearer {}", token.as_str()),
                                url: url_str,
                                reader: &mut reader,
                                media_type: reader_mime_type.clone(),
                                content_length: size
                            }.upload().await
                        };
                        match upload_result {
                            None => {
                                dlg.finished(false);
                                return Err(client::Error::Cancelled)
                            }
                            Some(Err(err)) => {
                                dlg.finished(false);
                                return Err(client::Error::HttpError(err))
                            }
                            Some(Ok(upload_result)) => {
                                res = upload_result;
                                if !res.status().is_success() {
                                    dlg.store_upload_url(None);
                                    dlg.finished(false);
                                    return Err(client::Error::Failure(res))
                                }
                            }
                        }
                    }
                    let result_value = {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        match json::from_str(&res_body_string) {
                            Ok(decoded) => (res, decoded),
                            Err(err) => {
                                dlg.response_json_decode_error(&res_body_string, &err);
                                return Err(client::Error::JsonDecodeError(res_body_string, err));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(result_value)
                }
            }
        }
    }

    /// Upload media in a resumable fashion.
    /// Even if the upload fails or is interrupted, it can be resumed for a
    /// certain amount of time as the server maintains state temporarily.
    /// 
    /// The delegate will be asked for an `upload_url()`, and if not provided, will be asked to store an upload URL
    /// that was provided by the server, using `store_upload_url(...)`. The upload will be done in chunks, the delegate
    /// may specify the `chunk_size()` and may cancel the operation before each chunk is uploaded, using
    /// `cancel_chunk_upload(...)`.
    ///
    /// * *multipart*: yes
    /// * *max size*: 0kb
    /// * *valid mime types*: 'image/*' and 'video/*'
    pub async fn upload_resumable<RS>(self, resumeable_stream: RS, mime_type: mime::Mime) -> client::Result<(hyper::Response<hyper::body::Body>, Media)>
                where RS: client::ReadSeek {
        self.doit(resumeable_stream, mime_type, "resumable").await
    }
    /// Upload media all at once.
    /// If the upload fails for whichever reason, all progress is lost.
    ///
    /// * *multipart*: yes
    /// * *max size*: 0kb
    /// * *valid mime types*: 'image/*' and 'video/*'
    pub async fn upload<RS>(self, stream: RS, mime_type: mime::Mime) -> client::Result<(hyper::Response<hyper::body::Body>, Media)>
                where RS: client::ReadSeek {
        self.doit(stream, mime_type, "simple").await
    }

    ///
    /// Sets the *request* property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn request(mut self, new_value: Media) -> MediaInsertCall<'a> {
        self._request = new_value;
        self
    }
    /// The ID of the user to create the activity on behalf of.
    ///
    /// Sets the *user id* path property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn user_id(mut self, new_value: &str) -> MediaInsertCall<'a> {
        self._user_id = new_value.to_string();
        self
    }
    ///
    /// Sets the *collection* path property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn collection(mut self, new_value: &str) -> MediaInsertCall<'a> {
        self._collection = new_value.to_string();
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    /// 
    /// It should be used to handle progress information, and to implement a certain level of resilience.
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(mut self, new_value: &'a mut dyn client::Delegate) -> MediaInsertCall<'a> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *alt* (query-string) - Data format for the response.
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - An opaque string that represents a user for quota purposes. Must not exceed 40 characters.
    /// * *userIp* (query-string) - Deprecated. Please use quotaUser instead.
    pub fn param<T>(mut self, name: T, value: T) -> MediaInsertCall<'a>
                                                        where T: AsRef<str> {
        self._additional_params.insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead the default `Scope` variant
    /// `Scope::PluLogin`.
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    /// If `None` is specified, then all scopes will be removed and no default scope will be used either.
    /// In that case, you have to specify your API-key using the `key` parameter (see the `param()`
    /// function for details).
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<T, S>(mut self, scope: T) -> MediaInsertCall<'a>
                                                        where T: Into<Option<S>>,
                                                              S: AsRef<str> {
        match scope.into() {
          Some(scope) => self._scopes.insert(scope.as_ref().to_string(), ()),
          None => None,
        };
        self
    }
}


/// Get a person's profile.
///
/// A builder for the *get* method supported by a *people* resource.
/// It is not used directly, but through a `PeopleMethods` instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate yup_oauth2 as oauth2;
/// # extern crate google_plusdomains1 as plusdomains1;
/// # async fn dox() {
/// # use std::default::Default;
/// # use oauth2;
/// # use plusdomains1::PlusDomains;
/// 
/// # let secret: oauth2::ApplicationSecret = Default::default();
/// # let auth = yup_oauth2::InstalledFlowAuthenticator::builder(
/// #         secret,
/// #         yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     ).build().await.unwrap();
/// # let mut hub = PlusDomains::new(hyper::Client::builder().build(hyper_rustls::HttpsConnector::with_native_roots()), auth);
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.people().get("userId")
///              .doit().await;
/// # }
/// ```
pub struct PeopleGetCall<'a>
    where  {

    hub: &'a PlusDomains<>,
    _user_id: String,
    _delegate: Option<&'a mut dyn client::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeMap<String, ()>
}

impl<'a> client::CallBuilder for PeopleGetCall<'a> {}

impl<'a> PeopleGetCall<'a> {


    /// Perform the operation you have build so far.
    pub async fn doit(mut self) -> client::Result<(hyper::Response<hyper::body::Body>, Person)> {
        use std::io::{Read, Seek};
        use hyper::header::{CONTENT_TYPE, CONTENT_LENGTH, AUTHORIZATION, USER_AGENT, LOCATION};
        use client::ToParts;
        let mut dd = client::DefaultDelegate;
        let mut dlg: &mut dyn client::Delegate = match self._delegate {
            Some(d) => d,
            None => &mut dd
        };
        dlg.begin(client::MethodInfo { id: "plusDomains.people.get",
                               http_method: hyper::Method::GET });
        let mut params: Vec<(&str, String)> = Vec::with_capacity(3 + self._additional_params.len());
        params.push(("userId", self._user_id.to_string()));
        for &field in ["alt", "userId"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(client::Error::FieldClash(field));
            }
        }
        for (name, value) in self._additional_params.iter() {
            params.push((&name, value.clone()));
        }

        params.push(("alt", "json".to_string()));

        let mut url = self.hub._base_url.clone() + "people/{userId}";
        if self._scopes.len() == 0 {
            self._scopes.insert(Scope::PluLogin.as_ref().to_string(), ());
        }

        for &(find_this, param_name) in [("{userId}", "userId")].iter() {
            let mut replace_with: Option<&str> = None;
            for &(name, ref value) in params.iter() {
                if name == param_name {
                    replace_with = Some(value);
                    break;
                }
            }
            url = url.replace(find_this, replace_with.expect("to find substitution value in params"));
        }
        {
            let mut indices_for_removal: Vec<usize> = Vec::with_capacity(1);
            for param_name in ["userId"].iter() {
                if let Some(index) = params.iter().position(|t| &t.0 == param_name) {
                    indices_for_removal.push(index);
                }
            }
            for &index in indices_for_removal.iter() {
                params.remove(index);
            }
        }

        let url = url::Url::parse_with_params(&url, params).unwrap();



        loop {
            let token = match self.hub.auth.token(&self._scopes.keys().collect::<Vec<_>>()[..]).await {
                Ok(token) => token.clone(),
                Err(err) => {
                    match  dlg.token(&err) {
                        Some(token) => token,
                        None => {
                            dlg.finished(false);
                            return Err(client::Error::MissingToken(err))
                        }
                    }
                }
            };
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder().method(hyper::Method::GET).uri(url.clone().into_string())
                        .header(USER_AGENT, self.hub._user_agent.clone())                            .header(AUTHORIZATION, format!("Bearer {}", token.as_str()));


                        let request = req_builder
                        .body(hyper::body::Body::empty());

                client.request(request.unwrap()).await
                
            };

            match req_result {
                Err(err) => {
                    if let client::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d);
                        continue;
                    }
                    dlg.finished(false);
                    return Err(client::Error::HttpError(err))
                }
                Ok(mut res) => {
                    if !res.status().is_success() {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        let json_server_error = json::from_str::<client::JsonServerError>(&res_body_string).ok();
                        let server_error = json::from_str::<client::ServerError>(&res_body_string)
                            .or_else(|_| json::from_str::<client::ErrorResponse>(&res_body_string).map(|r| r.error))
                            .ok();

                        if let client::Retry::After(d) = dlg.http_failure(&res,
                                                              json_server_error,
                                                              server_error) {
                            sleep(d);
                            continue;
                        }
                        dlg.finished(false);
                        return match json::from_str::<client::ErrorResponse>(&res_body_string){
                            Err(_) => Err(client::Error::Failure(res)),
                            Ok(serr) => Err(client::Error::BadRequest(serr))
                        }
                    }
                    let result_value = {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        match json::from_str(&res_body_string) {
                            Ok(decoded) => (res, decoded),
                            Err(err) => {
                                dlg.response_json_decode_error(&res_body_string, &err);
                                return Err(client::Error::JsonDecodeError(res_body_string, err));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(result_value)
                }
            }
        }
    }


    /// The ID of the person to get the profile for. The special value "me" can be used to indicate the authenticated user.
    ///
    /// Sets the *user id* path property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn user_id(mut self, new_value: &str) -> PeopleGetCall<'a> {
        self._user_id = new_value.to_string();
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    /// 
    /// It should be used to handle progress information, and to implement a certain level of resilience.
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(mut self, new_value: &'a mut dyn client::Delegate) -> PeopleGetCall<'a> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *alt* (query-string) - Data format for the response.
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - An opaque string that represents a user for quota purposes. Must not exceed 40 characters.
    /// * *userIp* (query-string) - Deprecated. Please use quotaUser instead.
    pub fn param<T>(mut self, name: T, value: T) -> PeopleGetCall<'a>
                                                        where T: AsRef<str> {
        self._additional_params.insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead the default `Scope` variant
    /// `Scope::PluLogin`.
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    /// If `None` is specified, then all scopes will be removed and no default scope will be used either.
    /// In that case, you have to specify your API-key using the `key` parameter (see the `param()`
    /// function for details).
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<T, S>(mut self, scope: T) -> PeopleGetCall<'a>
                                                        where T: Into<Option<S>>,
                                                              S: AsRef<str> {
        match scope.into() {
          Some(scope) => self._scopes.insert(scope.as_ref().to_string(), ()),
          None => None,
        };
        self
    }
}


/// List all of the people in the specified collection.
///
/// A builder for the *list* method supported by a *people* resource.
/// It is not used directly, but through a `PeopleMethods` instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate yup_oauth2 as oauth2;
/// # extern crate google_plusdomains1 as plusdomains1;
/// # async fn dox() {
/// # use std::default::Default;
/// # use oauth2;
/// # use plusdomains1::PlusDomains;
/// 
/// # let secret: oauth2::ApplicationSecret = Default::default();
/// # let auth = yup_oauth2::InstalledFlowAuthenticator::builder(
/// #         secret,
/// #         yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     ).build().await.unwrap();
/// # let mut hub = PlusDomains::new(hyper::Client::builder().build(hyper_rustls::HttpsConnector::with_native_roots()), auth);
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.people().list("userId", "collection")
///              .page_token("ea")
///              .order_by("dolor")
///              .max_results(45)
///              .doit().await;
/// # }
/// ```
pub struct PeopleListCall<'a>
    where  {

    hub: &'a PlusDomains<>,
    _user_id: String,
    _collection: String,
    _page_token: Option<String>,
    _order_by: Option<String>,
    _max_results: Option<u32>,
    _delegate: Option<&'a mut dyn client::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeMap<String, ()>
}

impl<'a> client::CallBuilder for PeopleListCall<'a> {}

impl<'a> PeopleListCall<'a> {


    /// Perform the operation you have build so far.
    pub async fn doit(mut self) -> client::Result<(hyper::Response<hyper::body::Body>, PeopleFeed)> {
        use std::io::{Read, Seek};
        use hyper::header::{CONTENT_TYPE, CONTENT_LENGTH, AUTHORIZATION, USER_AGENT, LOCATION};
        use client::ToParts;
        let mut dd = client::DefaultDelegate;
        let mut dlg: &mut dyn client::Delegate = match self._delegate {
            Some(d) => d,
            None => &mut dd
        };
        dlg.begin(client::MethodInfo { id: "plusDomains.people.list",
                               http_method: hyper::Method::GET });
        let mut params: Vec<(&str, String)> = Vec::with_capacity(7 + self._additional_params.len());
        params.push(("userId", self._user_id.to_string()));
        params.push(("collection", self._collection.to_string()));
        if let Some(value) = self._page_token {
            params.push(("pageToken", value.to_string()));
        }
        if let Some(value) = self._order_by {
            params.push(("orderBy", value.to_string()));
        }
        if let Some(value) = self._max_results {
            params.push(("maxResults", value.to_string()));
        }
        for &field in ["alt", "userId", "collection", "pageToken", "orderBy", "maxResults"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(client::Error::FieldClash(field));
            }
        }
        for (name, value) in self._additional_params.iter() {
            params.push((&name, value.clone()));
        }

        params.push(("alt", "json".to_string()));

        let mut url = self.hub._base_url.clone() + "people/{userId}/people/{collection}";
        if self._scopes.len() == 0 {
            self._scopes.insert(Scope::PluCircleRead.as_ref().to_string(), ());
        }

        for &(find_this, param_name) in [("{userId}", "userId"), ("{collection}", "collection")].iter() {
            let mut replace_with: Option<&str> = None;
            for &(name, ref value) in params.iter() {
                if name == param_name {
                    replace_with = Some(value);
                    break;
                }
            }
            url = url.replace(find_this, replace_with.expect("to find substitution value in params"));
        }
        {
            let mut indices_for_removal: Vec<usize> = Vec::with_capacity(2);
            for param_name in ["collection", "userId"].iter() {
                if let Some(index) = params.iter().position(|t| &t.0 == param_name) {
                    indices_for_removal.push(index);
                }
            }
            for &index in indices_for_removal.iter() {
                params.remove(index);
            }
        }

        let url = url::Url::parse_with_params(&url, params).unwrap();



        loop {
            let token = match self.hub.auth.token(&self._scopes.keys().collect::<Vec<_>>()[..]).await {
                Ok(token) => token.clone(),
                Err(err) => {
                    match  dlg.token(&err) {
                        Some(token) => token,
                        None => {
                            dlg.finished(false);
                            return Err(client::Error::MissingToken(err))
                        }
                    }
                }
            };
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder().method(hyper::Method::GET).uri(url.clone().into_string())
                        .header(USER_AGENT, self.hub._user_agent.clone())                            .header(AUTHORIZATION, format!("Bearer {}", token.as_str()));


                        let request = req_builder
                        .body(hyper::body::Body::empty());

                client.request(request.unwrap()).await
                
            };

            match req_result {
                Err(err) => {
                    if let client::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d);
                        continue;
                    }
                    dlg.finished(false);
                    return Err(client::Error::HttpError(err))
                }
                Ok(mut res) => {
                    if !res.status().is_success() {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        let json_server_error = json::from_str::<client::JsonServerError>(&res_body_string).ok();
                        let server_error = json::from_str::<client::ServerError>(&res_body_string)
                            .or_else(|_| json::from_str::<client::ErrorResponse>(&res_body_string).map(|r| r.error))
                            .ok();

                        if let client::Retry::After(d) = dlg.http_failure(&res,
                                                              json_server_error,
                                                              server_error) {
                            sleep(d);
                            continue;
                        }
                        dlg.finished(false);
                        return match json::from_str::<client::ErrorResponse>(&res_body_string){
                            Err(_) => Err(client::Error::Failure(res)),
                            Ok(serr) => Err(client::Error::BadRequest(serr))
                        }
                    }
                    let result_value = {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        match json::from_str(&res_body_string) {
                            Ok(decoded) => (res, decoded),
                            Err(err) => {
                                dlg.response_json_decode_error(&res_body_string, &err);
                                return Err(client::Error::JsonDecodeError(res_body_string, err));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(result_value)
                }
            }
        }
    }


    /// Get the collection of people for the person identified. Use "me" to indicate the authenticated user.
    ///
    /// Sets the *user id* path property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn user_id(mut self, new_value: &str) -> PeopleListCall<'a> {
        self._user_id = new_value.to_string();
        self
    }
    /// The collection of people to list.
    ///
    /// Sets the *collection* path property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn collection(mut self, new_value: &str) -> PeopleListCall<'a> {
        self._collection = new_value.to_string();
        self
    }
    /// The continuation token, which is used to page through large result sets. To get the next page of results, set this parameter to the value of "nextPageToken" from the previous response.
    ///
    /// Sets the *page token* query property to the given value.
    pub fn page_token(mut self, new_value: &str) -> PeopleListCall<'a> {
        self._page_token = Some(new_value.to_string());
        self
    }
    /// The order to return people in.
    ///
    /// Sets the *order by* query property to the given value.
    pub fn order_by(mut self, new_value: &str) -> PeopleListCall<'a> {
        self._order_by = Some(new_value.to_string());
        self
    }
    /// The maximum number of people to include in the response, which is used for paging. For any response, the actual number returned might be less than the specified maxResults.
    ///
    /// Sets the *max results* query property to the given value.
    pub fn max_results(mut self, new_value: u32) -> PeopleListCall<'a> {
        self._max_results = Some(new_value);
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    /// 
    /// It should be used to handle progress information, and to implement a certain level of resilience.
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(mut self, new_value: &'a mut dyn client::Delegate) -> PeopleListCall<'a> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *alt* (query-string) - Data format for the response.
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - An opaque string that represents a user for quota purposes. Must not exceed 40 characters.
    /// * *userIp* (query-string) - Deprecated. Please use quotaUser instead.
    pub fn param<T>(mut self, name: T, value: T) -> PeopleListCall<'a>
                                                        where T: AsRef<str> {
        self._additional_params.insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead the default `Scope` variant
    /// `Scope::PluCircleRead`.
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    /// If `None` is specified, then all scopes will be removed and no default scope will be used either.
    /// In that case, you have to specify your API-key using the `key` parameter (see the `param()`
    /// function for details).
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<T, S>(mut self, scope: T) -> PeopleListCall<'a>
                                                        where T: Into<Option<S>>,
                                                              S: AsRef<str> {
        match scope.into() {
          Some(scope) => self._scopes.insert(scope.as_ref().to_string(), ()),
          None => None,
        };
        self
    }
}


/// Shut down. See https://developers.google.com/+/api-shutdown for more details.
///
/// A builder for the *listByActivity* method supported by a *people* resource.
/// It is not used directly, but through a `PeopleMethods` instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate yup_oauth2 as oauth2;
/// # extern crate google_plusdomains1 as plusdomains1;
/// # async fn dox() {
/// # use std::default::Default;
/// # use oauth2;
/// # use plusdomains1::PlusDomains;
/// 
/// # let secret: oauth2::ApplicationSecret = Default::default();
/// # let auth = yup_oauth2::InstalledFlowAuthenticator::builder(
/// #         secret,
/// #         yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     ).build().await.unwrap();
/// # let mut hub = PlusDomains::new(hyper::Client::builder().build(hyper_rustls::HttpsConnector::with_native_roots()), auth);
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.people().list_by_activity("activityId", "collection")
///              .page_token("sed")
///              .max_results(31)
///              .doit().await;
/// # }
/// ```
pub struct PeopleListByActivityCall<'a>
    where  {

    hub: &'a PlusDomains<>,
    _activity_id: String,
    _collection: String,
    _page_token: Option<String>,
    _max_results: Option<u32>,
    _delegate: Option<&'a mut dyn client::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeMap<String, ()>
}

impl<'a> client::CallBuilder for PeopleListByActivityCall<'a> {}

impl<'a> PeopleListByActivityCall<'a> {


    /// Perform the operation you have build so far.
    pub async fn doit(mut self) -> client::Result<(hyper::Response<hyper::body::Body>, PeopleFeed)> {
        use std::io::{Read, Seek};
        use hyper::header::{CONTENT_TYPE, CONTENT_LENGTH, AUTHORIZATION, USER_AGENT, LOCATION};
        use client::ToParts;
        let mut dd = client::DefaultDelegate;
        let mut dlg: &mut dyn client::Delegate = match self._delegate {
            Some(d) => d,
            None => &mut dd
        };
        dlg.begin(client::MethodInfo { id: "plusDomains.people.listByActivity",
                               http_method: hyper::Method::GET });
        let mut params: Vec<(&str, String)> = Vec::with_capacity(6 + self._additional_params.len());
        params.push(("activityId", self._activity_id.to_string()));
        params.push(("collection", self._collection.to_string()));
        if let Some(value) = self._page_token {
            params.push(("pageToken", value.to_string()));
        }
        if let Some(value) = self._max_results {
            params.push(("maxResults", value.to_string()));
        }
        for &field in ["alt", "activityId", "collection", "pageToken", "maxResults"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(client::Error::FieldClash(field));
            }
        }
        for (name, value) in self._additional_params.iter() {
            params.push((&name, value.clone()));
        }

        params.push(("alt", "json".to_string()));

        let mut url = self.hub._base_url.clone() + "activities/{activityId}/people/{collection}";
        if self._scopes.len() == 0 {
            self._scopes.insert(Scope::PluLogin.as_ref().to_string(), ());
        }

        for &(find_this, param_name) in [("{activityId}", "activityId"), ("{collection}", "collection")].iter() {
            let mut replace_with: Option<&str> = None;
            for &(name, ref value) in params.iter() {
                if name == param_name {
                    replace_with = Some(value);
                    break;
                }
            }
            url = url.replace(find_this, replace_with.expect("to find substitution value in params"));
        }
        {
            let mut indices_for_removal: Vec<usize> = Vec::with_capacity(2);
            for param_name in ["collection", "activityId"].iter() {
                if let Some(index) = params.iter().position(|t| &t.0 == param_name) {
                    indices_for_removal.push(index);
                }
            }
            for &index in indices_for_removal.iter() {
                params.remove(index);
            }
        }

        let url = url::Url::parse_with_params(&url, params).unwrap();



        loop {
            let token = match self.hub.auth.token(&self._scopes.keys().collect::<Vec<_>>()[..]).await {
                Ok(token) => token.clone(),
                Err(err) => {
                    match  dlg.token(&err) {
                        Some(token) => token,
                        None => {
                            dlg.finished(false);
                            return Err(client::Error::MissingToken(err))
                        }
                    }
                }
            };
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder().method(hyper::Method::GET).uri(url.clone().into_string())
                        .header(USER_AGENT, self.hub._user_agent.clone())                            .header(AUTHORIZATION, format!("Bearer {}", token.as_str()));


                        let request = req_builder
                        .body(hyper::body::Body::empty());

                client.request(request.unwrap()).await
                
            };

            match req_result {
                Err(err) => {
                    if let client::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d);
                        continue;
                    }
                    dlg.finished(false);
                    return Err(client::Error::HttpError(err))
                }
                Ok(mut res) => {
                    if !res.status().is_success() {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        let json_server_error = json::from_str::<client::JsonServerError>(&res_body_string).ok();
                        let server_error = json::from_str::<client::ServerError>(&res_body_string)
                            .or_else(|_| json::from_str::<client::ErrorResponse>(&res_body_string).map(|r| r.error))
                            .ok();

                        if let client::Retry::After(d) = dlg.http_failure(&res,
                                                              json_server_error,
                                                              server_error) {
                            sleep(d);
                            continue;
                        }
                        dlg.finished(false);
                        return match json::from_str::<client::ErrorResponse>(&res_body_string){
                            Err(_) => Err(client::Error::Failure(res)),
                            Ok(serr) => Err(client::Error::BadRequest(serr))
                        }
                    }
                    let result_value = {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        match json::from_str(&res_body_string) {
                            Ok(decoded) => (res, decoded),
                            Err(err) => {
                                dlg.response_json_decode_error(&res_body_string, &err);
                                return Err(client::Error::JsonDecodeError(res_body_string, err));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(result_value)
                }
            }
        }
    }


    /// The ID of the activity to get the list of people for.
    ///
    /// Sets the *activity id* path property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn activity_id(mut self, new_value: &str) -> PeopleListByActivityCall<'a> {
        self._activity_id = new_value.to_string();
        self
    }
    /// The collection of people to list.
    ///
    /// Sets the *collection* path property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn collection(mut self, new_value: &str) -> PeopleListByActivityCall<'a> {
        self._collection = new_value.to_string();
        self
    }
    /// The continuation token, which is used to page through large result sets. To get the next page of results, set this parameter to the value of "nextPageToken" from the previous response.
    ///
    /// Sets the *page token* query property to the given value.
    pub fn page_token(mut self, new_value: &str) -> PeopleListByActivityCall<'a> {
        self._page_token = Some(new_value.to_string());
        self
    }
    /// The maximum number of people to include in the response, which is used for paging. For any response, the actual number returned might be less than the specified maxResults.
    ///
    /// Sets the *max results* query property to the given value.
    pub fn max_results(mut self, new_value: u32) -> PeopleListByActivityCall<'a> {
        self._max_results = Some(new_value);
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    /// 
    /// It should be used to handle progress information, and to implement a certain level of resilience.
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(mut self, new_value: &'a mut dyn client::Delegate) -> PeopleListByActivityCall<'a> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *alt* (query-string) - Data format for the response.
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - An opaque string that represents a user for quota purposes. Must not exceed 40 characters.
    /// * *userIp* (query-string) - Deprecated. Please use quotaUser instead.
    pub fn param<T>(mut self, name: T, value: T) -> PeopleListByActivityCall<'a>
                                                        where T: AsRef<str> {
        self._additional_params.insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead the default `Scope` variant
    /// `Scope::PluLogin`.
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    /// If `None` is specified, then all scopes will be removed and no default scope will be used either.
    /// In that case, you have to specify your API-key using the `key` parameter (see the `param()`
    /// function for details).
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<T, S>(mut self, scope: T) -> PeopleListByActivityCall<'a>
                                                        where T: Into<Option<S>>,
                                                              S: AsRef<str> {
        match scope.into() {
          Some(scope) => self._scopes.insert(scope.as_ref().to_string(), ()),
          None => None,
        };
        self
    }
}


