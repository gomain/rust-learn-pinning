use std::marker::PhantomPinned;
use std::pin::Pin;

#[derive(Debug)]
pub struct Country {
    people: Vec<Pin<Box<Person>>>,
    king: *const Person,
}

#[derive(Debug)]
pub struct Person(String, PhantomPinned);

impl Country {
    pub fn new() -> Self {
        Self {
            people: Vec::new(),
            king: std::ptr::null(),
        }
    }
    pub fn enter(&mut self, name: String) {
        self.people.push(Person::new(name));
    }

    pub fn crown(&mut self, name: String) {
        if let Some(king) = self.people.iter().find(|p| p.0 == name) {
            self.king = king.as_ref().get_ref();
        } else {
            let king = Person::new(name);
            self.king = king.as_ref().get_ref();
            self.people.push(king);
        }
    }
    // it's ok to return &Person, no need of Pin<&Person> because shared references are safe.
    // the returned king has the liftetime of his country.
    pub fn king(&self) -> Option<&Person> {
        let king = self.king; // raw pointers are Copy
        self.people
            .iter()
            .find(|person| king == person.as_ref().get_ref()) // Option<&Pin<Box<Person>>>
            .map(Pin::as_ref) // Pin<Box<Person>> ---> <Pin<&Person>> // Box ---> &
            .map(Pin::get_ref) // Pin<&Person> ---> &Person // Pin ---> _
    }
    // returns `Pin<&mut Person>` rather than `&mut Person` to guard against the king being moved!.
    // the returned king has the liftetime of his country.
    pub fn mutable_king(&mut self) -> Option<Pin<&mut Person>> {
        let king = self.king; // raw pointers are Copy
        self.people
            .iter_mut()
            .find(|person| king == person.as_ref().get_ref()) // Option<&mut Pin<Box<Person>>>
            .map(Pin::as_mut)
    }
}

impl Person {
    pub fn new(name: String) -> Pin<Box<Self>> {
        Box::pin(Self(name, PhantomPinned))
    }

    pub fn name(&self) -> &String {
        &self.0
    }

    pub fn rename(self: Pin<&mut Self>, new_name: String) {
        // Safety: we know we aren't moving the person here.
        unsafe {
            self.get_unchecked_mut().0 = new_name;
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn play() {
        let mut country = Country::new();
        country.enter("peter".to_string());
        country.crown("gomain".to_string());
        country.enter("john".to_string());
        dbg!(&country);

        dbg!(country.king().unwrap().name());

        country.mutable_king().unwrap().rename("joe".to_string());
        country.crown("john".to_string());
        dbg!(country.king().unwrap().name());
        dbg!(&country);

        country.crown("joe".to_string());
        dbg!(&country);
    }
}
