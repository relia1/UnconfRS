# User Guide

[<- Back to Main README](../README.md) | [Setup Guide](SETUP.md)

This guide covers how to use this application based on your role

## For Users (Session Submitters and Voters)

### Registration

1. Navigate to the Login page
2. Click Register New User
3. Fill out first name, last name, email address, password, and password confirmation
4. Click submit

### Login

1. Navigate to the Login page
2. Fill out email and password
3. Click submit (redirects to sessions page)

### Logout

1. A logged-in user will have a Logout on the far right of the nav bar
2. Click Logout (redirects to login page)

### View and Interacting with Sessions

The sessions page displays all submitted sessions in an interactive table with these features:

- Submission: A User os able to submit a new session
- Edit: A User is able to edit their own submitted sessions
- Delete: A User is able to delete their own submitted sessions
- Voting: A User is able to vote on sessions they would like to attend
- Table Specific Features:
    - Search: A User can use the search box to search the subbmitted sessions
    - Sort: A User can sort based on the various column headers
    - Pagination: A User can click through the pages and even change the entries per page

### View and Interact with the Schedule

The unconf_schedule page will display a schedule with the rooms and timeslots with a user being able to:

- View: Get an overall idea of what rooms and times sessions will be taking place
- Update view type: By default the "Time-Based" view of the schedule will have rooms as the column headers and time as
  the row headers. If a User changes this to a "Room-Based" view the times are the column headers and the room names are
  the row headers instead.

## For Facilitators (All things a User can do but with some additional functionality)

Facilitators in addition to being able to do what a User can do will be able to make changes to sessions even if they
were not the creator of that session.

## For Admin (All things a Facilitator can do but with some additional functionality)

Admin in addition to being able to do what a Facilitator can do will be able to create rooms, timeslots, and interact
with the scheduling related features.

### Setting up the Unconference

Once an admin navigates to the schedule page when rooms/timeslots haven't been created, they will have a form they will
fill out for configuring the unconference rooms and timeslots.

1. Setting up the rooms
    1. Click add a room
    2. Fill out room name and location information
    3. Repeat for each room
    4. Click Save Rooms
2. Settings up the timeslots
    1. Click Add Timeslot
    2. Add a starting time and duration, you can also mark this timeslot as non schedulable by checking the Blocked
       checkbox and providing a text reason.
    3. Repeat for each timeslot
    4. Click Save Timeslots
3. Adding rooms later
    1. On the schedule page click on Add Room
    2. Fill out room name and location
    3. Click Save
4. Adding timeslots later
    1. Click Timeslot Configuration
    2. You'll see greyed out timeslots representing existing timeslots
    3. Click Add Timeslot to add more
    4. Fill out the start time and duration
    5. Click Save Timeslots
5. Clicking Populate Schedule will populate the schedule with an attempted "Best Schedule"
6. Clicking clear schedule will remove all sessions from the schedule
7. Once sessions exist on the schedule you can manually drag and drop them to different places

---

**Related Documentation:**

- [Development](DEVELOPMENT.md) - Technical details and structure
- [Architecture](ARCHITECTURE.md) - Technologies and system design