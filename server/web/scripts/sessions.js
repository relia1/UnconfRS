const {get, set, update} = idbKeyval;
set('sessions_voted_for', current_users_voted_sessions);

let currentUserId = null;
let currentSessionId = null;
let currentTagId  = null;

document.addEventListener('DOMContentLoaded', function() {
    let table = new DataTable('.sessionsTable', {
        columns:   [
            {
                data:      null,
                className: 'dt-control',
                defaultContent: '',
                orderable: false,
            },
            {data: 'session_id', visible: false},
            {data: 'title'},
            {data: 'name'},
            {data: 'email'},
            {data: 'tags'},
            {
                data:      null,
                defaultContent: '<button class="del-btn btn btn-danger btn-sm me-1">Delete</button>' +
                                    '<button class="edit-btn btn btn-primary btn-sm me-1">Edit</button>' +
                                    '<button class="upvote-btn btn btn-success btn-sm">Upvote</button>',
                orderable: false,
            },
            {data: 'content', visible: false},
            {data: 'user_id', visible: false},
        ],
        searching: true,
        ordering:  true,
        paging:    true,
        responsive: true,
        order:     [[1, 'asc']],
    });

    // Add event listener for opening and closing details
    table.on('click', 'td.dt-control', function(e) {
        let tr = e.target.closest('tr');
        let row = table.row(tr);

        if (row.child.isShown()) {
            // This row is already open - close it
            row.child.hide();
        } else {
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
                    method:  'DELETE',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                });

                if (!response.ok) {
                    const error = await response.json();
                    throw new Error(error.error);
                }

                /* Session was deleted successfully from the database so now also remove
                 it from the table */
                row.remove().draw(false);
            } catch (error) {
                console.error('Error deleting session:', error);
                if (error.message.match(/foreign key constraint/)) {
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

        // Populate form with existing data
        document.getElementById('title').value          = data.title;
        document.getElementById('sessionContent').value = data.content;

        // Set current tag in dropdown
        const tagSelect = document.getElementById('tagSelect');
        if (data.tags === '') {
            tagSelect.value = '';
            currentTagId    = null;
        } else {
            currentTagId    =
                parseInt(Array.from(tagSelect.options).find(opt => opt.text === data.tags).value);
            tagSelect.value = currentTagId;
        }

        const modal = new bootstrap.Modal(document.getElementById('sessionModal'));
        modal.show();
    });

    table.on('click', '.upvote-btn', async function(e) {
        var data = table.row($(this).closest('tr')).data();
        currentSessionId = Number(data.session_id);
        currentUserId    = Number(data.user_id);
        let response;

        if (!await hasVoted(currentSessionId)) {
            try {
                response = await fetch(`/api/v1/sessions/${currentSessionId}/increment`, {
                    method:  'PUT',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                });

                if (!response.ok) {
                    const error = await response.json();
                    throw new Error(error.error);
                }

                alert('Session upvoted successfully');
                await setVotesVal(currentSessionId);
            } catch (error) {
                console.error('Error upvoting session:', error);
                alert('There was an error upvoting the session. Please try again.');
            }
        } else {
            try {
                response = await fetch(`/api/v1/sessions/${currentSessionId}/decrement`, {
                    method:  'PUT',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                });

                if (!response.ok) {
                    const error = await response.json();
                    throw new Error(error.error);
                }

                alert('Session unvoted successfully');
                await setVotesVal(currentSessionId);
            } catch (error) {
                console.error('Error removing vote for session:', error);
                alert('There was an error removing the vote for the session. Please try again.');
            }
        }
    });

    document.getElementById('sessionForm').addEventListener('submit', async function(event) {
        event.preventDefault();
        const title    = document.getElementById('title').value;
        const content  = document.getElementById('sessionContent').value;
        const newTagId = parseInt(document.getElementById('tagSelect').value) ?? null;
        const isEdit   = currentSessionId !== null;

        let response;
        if (isEdit) {
            try {
                // Update session title and content
                response = await fetch(`/api/v1/sessions/${currentSessionId}`, {
                    method:  'PUT',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({user_id: currentUserId, title, content}),
                });

                if (!response.ok) {
                    const error = await response.json();
                    throw new Error(error.error);
                }

                // Handle tag changes
                if (newTagId !== currentTagId) {
                    let tagResponse;
                    if (newTagId && currentTagId) {
                        // Update existing tag to new tag
                        tagResponse = await fetch(`/api/v1/sessions/${currentSessionId}/tags`, {
                            method:  'PUT',
                            headers: {
                                'Content-Type': 'application/json',
                            },
                            body:    JSON.stringify({
                                old_tag_id: currentTagId,
                                new_tag_id: newTagId,
                            }),
                        });
                    } else if (currentTagId && !newTagId) {
                        // Remove tag when "No tag" is selected
                        tagResponse = await fetch(`/api/v1/sessions/${currentSessionId}/tags`, {
                            method:  'DELETE',
                            headers: {
                                'Content-Type': 'application/json',
                            },
                            body:    JSON.stringify({tag_id: currentTagId}),
                        });
                    } else if (!currentTagId && newTagId) {
                        // Add new tag when session had no tag before
                        tagResponse = await fetch(`/api/v1/sessions/${currentSessionId}/tags`, {
                            method:  'POST',
                            headers: {
                                'Content-Type': 'application/json',
                            },
                            body:    JSON.stringify({tag_id: newTagId}),
                        });
                    }

                    if (tagResponse && !tagResponse.ok) {
                        const tagError = await tagResponse.json();
                        console.error('Error updating tag:', tagError);
                        alert('There was an error updating the tag. Please try again.');
                        return;
                    }
                }

                alert('Session updated successfully!');
                bootstrap.Modal.getInstance(document.getElementById('sessionModal')).hide();
            } catch (error) {
                console.log('Error updating session: ', error);
                alert('There was an error updating the session. Please try again.');
            }
            location.reload();
        } else {
            try {
                const requestBody = {title, content};
                if (newTagId) {
                    requestBody.tag_id = parseInt(newTagId);
                }

                response = await fetch('/api/v1/sessions/add', {
                    method:  'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify(requestBody),
                });

                if (!response.ok) {
                    throw new Error(`HTTP error! status: ${response.status}`);
                }

                alert('Session submitted successfully!');
                bootstrap.Modal.getInstance(document.getElementById('sessionModal')).hide();
            } catch (error) {
                console.log('Error submitting session: ', error);
                alert('There was an error submitting the session. Please try again.');
                return;
            }
            location.reload();
        }
    });

    {% if is_authenticated %}
        document.querySelector('#add-session').addEventListener('click', async function(data) {
            const modal = new bootstrap.Modal(document.getElementById('sessionModal'));
            document.getElementById('sessionForm').reset();
            currentSessionId = null;
            currentUserId    = null;
            currentTagId     = null;
            modal.show();
        });
    {% endif %}
});

function format(data) {
    // data is the original data object for the row
    return (
        '<dl>' + '<dt><b>Session Description</b></dt>' + '<dd>' + data.content + '</dd>' + '</dl>'
    );
}


/* Voting functions */
async function hasVoted(sessionId) {
    try {
        const sessionsUserVotedFor = await get('sessions_voted_for');
        if (!sessionsUserVotedFor) {
            return false;
        }
        return sessionsUserVotedFor.includes(sessionId);
    } catch (error) {
        console.error('Unable to read sessions_voted_for', error);
        return false;
    }
}

async function setVotesVal(sessionId) {
    try {
        let sessionsUserVotedFor = await get('sessions_voted_for') || [];
        if (!sessionsUserVotedFor || !sessionsUserVotedFor.includes(sessionId)) {
            sessionsUserVotedFor.push(sessionId);
            await set('sessions_voted_for', sessionsUserVotedFor);
            return true;
        } else {
            sessionsUserVotedFor = sessionsUserVotedFor.filter(val => val !== sessionId);
            await set('sessions_voted_for', sessionsUserVotedFor);
            return true;
        }
    } catch (error) {
        console.error('Unable to set sessions_voted_for', error);
        return false;
    }
}
