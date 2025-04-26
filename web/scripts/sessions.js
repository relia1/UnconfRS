let currentUserId = null;
let currentSessionId = null;

document.addEventListener('DOMContentLoaded', function () {
    let table = new DataTable('.sessionsTable', {
        columns: [
            {
                data: null,
                className: 'dt-control',
                defaultContent: '',
                orderable: false
            },
            {data: 'session_id', visible: false},
            {data: 'title'},
            {data: 'name'},
            {data: 'email'},
            {
                data: null,
                defaultContent: '<button class="del-btn btn-action"' +
                                    ' id="deleteSessionButton">Delete</button><button class="edit-btn btn-action"' +
                                    ' id="editSessionButton">Edit</button><button class="upvote-btn btn-action"' +
                                    ' id="upvoteSessionButton">Upvote</button>',
                orderable: false
            },
            {data: 'content', visible: false},
            {data: 'user_id', visible: false},
        ],
        searching: true,
        ordering: true,
        paging: true,
        responsive: true,
        order: [[1, 'asc']],
    });

    // Add event listener for opening and closing details
    table.on('click', 'td.dt-control', function (e) {
        let tr = e.target.closest('tr');
        let row = table.row(tr);

        if (row.child.isShown()) {
            // This row is already open - close it
            row.child.hide();
        }
        else {
            // Open this row
            row.child(format(row.data())).show();
        }
    });

    table.on('click', '.del-btn', async function(e) {
        if (confirm('Are you sure you want to delete this session?')) {
            let row = table.row($(this).closest('tr'));
            let data = row.data();
            console.log('Deleting session with id: ' + data.session_id);
            try {
                const response = await fetch(`/api/v1/sessions/${data.session_id}`, {
                    method: 'DELETE',
                    headers: {
                        'Content-Type': 'application/json'
                    },
                });

                if(!response.ok) {
                    const error = await response.json();
                    throw new Error(error.error);
                }

                /* Session was deleted successfully from the database so now also remove
                 it from the table */
                row.remove().draw(false);
            } catch (error) {
                console.error('Error deleting session:', error);
                if(error.message.match(/foreign key constraint/)) {
                    alert('This session cannot be deleted because it is associated with a' +
                        ' schedule session.');
                } else if (error.message.match(/Session does not belong to user/)) {
                    alert('Users can only delete sessions they have submitted');
                } else {
                    alert('There was an error deleting the session. Please try again.');
                }
            }
        }
    });

    table.on('click', '.edit-btn', async function(e) {
        var data = table.row($(this).closest('tr')).data();
        currentSessionId = data.session_id;
        currentUserId = data.user_id;
        console.log('Editing session with id: ' + data.session_id);
        showPopup(true, data);
    });

    table.on('click', '.upvote-btn', async function(e) {
        var data = table.row($(this).closest('tr')).data();
        currentSessionId = data.session_id;
        currentUserId = data.user_id;

        if (!await hasVoted(data.session_id)) {
            try {
                const response = await fetch(`/api/v1/sessions/${data.session_id}/increment`, {
                    method: 'PUT',
                    headers: {
                        'Content-Type': 'application/json'
                    },
                })
                .then(() => alert('Session upvoted successfully!'))
                    .catch(error => console.error('Error:', error));

                await setVotesCookie(data.session_id);
            } catch (error) {
                console.error('Error upvoting session:', error);
                alert('There was an error upvoting the session. Please try again.');
            }
        } else {
            alert('You have already upvoted this session!');
        }
    });

    document.getElementById('sessionForm').addEventListener('submit', async function(event) {
        event.preventDefault();
        const title = document.getElementById('title').value;
        const content = document.getElementById('sessionContent').value;
        const isEdit  = currentSessionId !== null;

        let response;
        if (isEdit) {
            try {
                await fetch(`/api/v1/sessions/${currentSessionId}`, {
                    method:  'PUT',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body:    JSON.stringify({user_id, title, content}),
                })
                .then(() => alert('Session updated successfully!'))
                .catch(error => console.error('Error:', error));
            } catch (error) {
                console.log('Error updating session: ', error);
            }
            location.reload();
        } else {
            try {
                response = await fetch('/api/v1/sessions/add', {
                    method:  'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body:    JSON.stringify({title, content}),
                });

                if (!response.ok) {
                    throw new Error(`HTTP error! status: ${response.status}`);
                }
            } catch (error) {
                console.log('Error submitting session: ', error);
            }
            location.reload();
        }
    });

    table.on('click', '.upvote-btn', async function(e) {
        var data = table.row($(this).closest('tr')).data();
        console.log('Upvoting session with id: ' + data.session_id);
    });

    document.querySelector('#add-session').addEventListener('click', async function(data) {
        await showPopup(false);
    });
});

function format(data) {
    // data is the original data object for the row
    return (
        '<dl>' + '<dt><b>Session Description</b></dt>' + '<dd>' + data.content + '</dd>' + '</dl>'
    );
}

/* Popup functions */
async function showPopup(isEdit, data=null) {
    const popup = document.getElementById('popup');
    const overlay = document.getElementById('overlay');
    popup.style.display = 'block';
    overlay.style.display = 'block';

    if(isEdit && data) {
        document.getElementById('title').value = data.title;
        document.getElementById('sessionContent').value = data.content;
        currentSessionId                                = data.session_id;
        currentUserId = data.user_id;
    } else {
        document.getElementById('sessionForm').reset();
        currentSessionId = null;
        currentUserId = null;
    }

    document.querySelector('#cancelButton').addEventListener('click', closePopup);
    document.querySelector('#overlay').addEventListener('click', closePopup);
}

function closePopup() {
    const popup = document.getElementById('popup');
    const overlay = document.getElementById('overlay');
    popup.style.display = 'none';
    overlay.style.display = 'none';function closePopup() {
    const popup = document.getElementById('popup');
    const overlay = document.getElementById('overlay');
    popup.style.display = 'none';
    overlay.style.display = 'none';

    document.querySelector('#cancelButton').removeEventListener('click', closePopup);
    document.querySelector('#overlay').removeEventListener('click', closePopup);
}

    document.querySelector('#cancelButton').removeEventListener('click', closePopup);
    document.querySelector('#overlay').removeEventListener('click', closePopup);
}

/* Voting functions */
async function hasVoted(sessionId) {
    votesCookie = await cookieStore.get('votes');
    return votesCookie?.value.split(',').includes(String(sessionId));
}

async function setVotesCookie(sessionId) {
    votesCookie = await cookieStore.get('votes');
    if(!votesCookie) {
        cookieStore.set('votes', String(sessionId));
    } else {
        cookieStore.set('votes', votesCookie.value.concat(',' + String(sessionId)));
    }
}