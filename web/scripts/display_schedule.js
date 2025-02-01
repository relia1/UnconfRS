let numOfTimeslots = 0;
let numOfRooms = 0;
let scheduleContainer = null;
let displayControls = null;
let draggedEvent = null;

document.addEventListener('DOMContentLoaded', function () {
    if (localStorage.getItem('admin') === 'true') {
        document.documentElement.className += ' admin';
    }

    scheduleContainer = document.querySelector('.schedule-container');
    displayControls = document.getElementById('controls');
    let timeslots = window.APP.timeslots;
    let rooms = window.APP.rooms;
    let events = window.APP.events;

    numOfRooms = rooms.length;
    numOfTimeslots = timeslots.length;

    function createEventBlock(event, displayType) {
        const eventBlock = document.createElement('div');
        eventBlock.className = 'event-block';
        eventBlock.textContent = event.title;
        eventBlock.draggable = true;

        // Set attributes for event block
        const attributes = {
            'data-start-time':  event.startTime.substring(0, 5),
            'data-end-time':    event.endTime.substring(0, 5),
            'data-timeslot-id': event.timeslotId,
            'data-topic-id':    event.topicId,
            'data-speaker-id':  event.speakerId,
            'data-schedule-id': event.scheduleId,
            'data-room-id':     event.roomId,
        };

        Object.entries(attributes).forEach(([attr, value]) => {
            eventBlock.setAttribute(attr, value);
        });

        eventBlock.addEventListener('dragstart', handleDragStart);

        const {top, height} = displayType === 'time' ?
            calculateEventPosition(attributes["data-start-time"]) :
            calculateRoomBasedEventPosition(attributes["data-room-id"]);

        eventBlock.style.top = top;
        eventBlock.style.height = height;

        return eventBlock;
    }

    function calculatePosition(index, total) {
        const multiplier = (
                               1 /
                               (
                                   total + 1
                               )
                           ) * 100;
        const top = (
                        (
                            index + 1
                        ) * multiplier
                    ) + '%';
        const height = multiplier + '%';
        return {top, height};
    }

    function calculateEventPosition(startTime) {
        const startIndex = timeslots.findIndex(slot => slot.start.substring(0, 5) === startTime);

        if (startIndex === -1) {
            return {top: '0%', height: '0%'};
        } else {
            return calculatePosition(startIndex, numOfTimeslots);
        }
    }

    function calculateRoomBasedEventPosition(roomId) {
        return calculatePosition(roomId - 1, numOfRooms);
    }


    function handleDragStart(event) {
        if (localStorage.getItem('admin') === 'true') {
            draggedEvent = event.target;
            event.dataTransfer.effectAllowed = "move";
            event.dataTransfer.setData("text/plain", null);
        } else {
            console.log('You do not have permission to move events');
        }
    }

    function handleDragOver(event) {
        event.preventDefault();
        event.dataTransfer.dropEffect = "move";
    }

    async function handleDrop(event) {
        event.preventDefault();
        if (!draggedEvent) {
            return;
        }

        const column = event.target.closest('.column');
        if (!column) {
            return;
        }

        const view = document.getElementById('view-selector').value;
        const rect = column.getBoundingClientRect();
        const relativePosition = (
                                     event.clientY - rect.top
                                 ) / rect.height;

        // Calculate new position data
        let timeslotIndex;
        if (view === 'time') {
            timeslotIndex =
                Math.floor(relativePosition *
                           (
                               numOfTimeslots + 1
                           )) - 1;
        } else {
            const columnTime = column.getAttribute('data-time');
            timeslotIndex = timeslots.findIndex(slot => slot.start.substring(0, 5) === columnTime);
        }

        if (timeslotIndex < 0 || timeslotIndex >= numOfTimeslots) {
            return;
        }

        const oldTimeslotId = Number(draggedEvent.getAttribute('data-timeslot-id'));
        const oldRoomId = draggedEvent.getAttribute('data-room-id');

        let newData = {
            timeslotId: timeslots[timeslotIndex].id,
            startTime:  timeslots[timeslotIndex].start.substring(0, 5),
            endTime:    timeslots[timeslotIndex].end.substring(0, 5),
            roomId:     view === 'time' ?
                            column.getAttribute('data-room-id') :
                            rooms[Math.floor(relativePosition *
                                             (
                                                 numOfRooms + 1
                                             )) - 1].id
        };

        if (!newData.roomId) {
            return;
        }

        // Update dragged event attributes
        Object.entries({
                           'data-start-time':  newData.startTime.substring(0, 5),
                           'data-end-time':    newData.endTime.substring(0, 5),
                           'data-room-id':     newData.roomId,
                           'data-timeslot-id': newData.timeslotId
                       }).forEach(([attr, value]) => {
            draggedEvent.setAttribute(attr, value);
        });

        // Update position
        const {top, height} = view === 'time' ?
            calculateEventPosition(newData.startTime.substring(0, 5)) :
            calculateRoomBasedEventPosition(newData.roomId);

        draggedEvent.style.top = top;
        draggedEvent.style.height = height;

        column.appendChild(draggedEvent);

        // Update the event in the backend
        try {
            const response = await fetch(`api/v1/timeslots/${oldTimeslotId}`, {
                method:  'PUT',
                headers: {'Content-Type': 'application/json'},
                body:    JSON.stringify({
                                            id:          newData.timeslotId,
                                            start_time:  newData.startTime,
                                            end_time:    newData.endTime,
                                            speaker_id:  Number(draggedEvent.getAttribute('data-speaker-id')),
                                            topic_id:    Number(draggedEvent.getAttribute('data-topic-id')),
                                            room_id:     Number(newData.roomId),
                                            old_room_id: Number(oldRoomId),
                                            schedule_id: Number(draggedEvent.getAttribute('data-schedule-id'))
                                        })
            });

            if (!response.ok) {
                if (response.status === 401) {
                    alert('You do not have permission to move events');
                } else {
                    alert('There was an error updating the schedule. Please try again.');
                }
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            // Update local events array
            const eventIndex = events.findIndex(e => e.timeslotId === oldTimeslotId);
            if (eventIndex !== -1) {
                events[eventIndex] = {...events[eventIndex], ...newData};
            }
        } catch (error) {
            console.error('Error updating the schedule:', error);
            alert('There was an error updating the schedule.');
        }

        draggedEvent = null;
    }

    document.getElementById('populate-schedule').addEventListener('click', async () => {
        try {
            const response = await fetch('/api/v1/schedules/generate', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
            });

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            location.reload();
        } catch (error) {
            console.error('Error populating schedule:', error);
            alert('There was an error populating the schedule. Please try again.');
        }
    });

    document.getElementById('clear-schedule').addEventListener('click', async () => {
        try {
            const response = await fetch('/api/v1/schedules/clear', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
            });

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            location.reload();
        } catch (error) {
            console.error('Error clearing schedule:', error);
            alert('There was an error clearing the schedule. Please try again.');
        }
    });

    document.getElementById('add-room').addEventListener('click', async () => {
        const popup = document.getElementById('popup');
        const overlay = document.getElementById('overlay');
        popup.style.display = 'block';
        overlay.style.display = 'block';
        document.querySelector('#cancelButton').addEventListener('click', closePopup);
        document.querySelector('#overlay').addEventListener('click', closePopup);
    });

    document.getElementById('submit-btn').addEventListener('click', async (e) => {
        e.preventDefault();
        const createRoomsForm = {
            rooms: [
                {
                    name:            document.getElementById('room-name').value,
                    location:        document.getElementById('room-location').value,
                    available_spots: Number(20)
                }
            ]
        };
        try {
            const response = await fetch('/api/v1/rooms/add', {
                method:  'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body:    JSON.stringify(createRoomsForm)
            });
            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }
            closePopup();
            location.reload();
        } catch (error) {
            console.error('Error submitting room:', error);
            alert('There was an error submitting the room. Please try again.');
        }
    });

    function closePopup() {
        const popup = document.getElementById('popup');
        const overlay = document.getElementById('overlay');
        popup.style.display = 'none';
        overlay.style.display = 'none';
        document.querySelector('#cancelButton').removeEventListener('click', closePopup);
        document.querySelector('#overlay').removeEventListener('click', closePopup);
    }

    async function removeRoom(roomId) {
        try {
            const response = await fetch(`/api/v1/rooms/${roomId}`, {
                method: 'DELETE',
                headers: {
                    'Content-Type': 'application/json'
                },
            });

            if (response.status === 401) {
                alert('You do not have permission to delete rooms');
                return;
            }

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            // Remove the room from the rooms array
            const roomIndex = rooms.findIndex(room => room.id === Number(roomId));
            if (roomIndex !== -1) {
                rooms.splice(roomIndex, 1);
            }

            numOfRooms = rooms.length;

            // Reload page
            location.reload();

        } catch (error) {
            console.error('Error deleting room:', error);
            alert('There was an error deleting the room. Please try again.');
        }
    }

    document.addEventListener('click', async function(e) {
        if (e.target && e.target.classList.contains('delete-room-btn')) {
            if (localStorage.getItem('admin') === 'true') {
                if (confirm('Are you sure you want to delete this room?')) {
                    const roomId = e.target.getAttribute('data-room-id');
                    await removeRoom(roomId)
                }
            } else {
                alert('You do not have permission to delete rooms');
            }
        }
    });


    function updateView() {
        const view = document.getElementById('view-selector').value;
        view === 'time' ? generateTimeBasedView() : generateRoomBasedView();
    }

    function generateTimeBasedView() {
        scheduleContainer.innerHTML = '';
        let timeSelector = document.getElementById('time-selector');
        if (timeSelector) {
            timeSelector.remove();
            document.getElementById('time-selector-label').remove();
        }

        // Create time slots column
        let rowColumn = document.createElement('div');
        rowColumn.className = 'row-column';
        let rowColumnHeader = document.createElement('div');
        rowColumnHeader.className = 'column-header';
        rowColumnHeader.innerText = 'Time';
        rowColumn.appendChild(rowColumnHeader);

        // Add time labels based on actual timeslots
        timeslots.forEach(slot => {
            const rowLabel = document.createElement('div');
            rowLabel.className = 'row-label';
            rowLabel.textContent = slot.start.substring(0, 5);
            rowColumn.appendChild(rowLabel);
        });

        scheduleContainer.appendChild(rowColumn);

        // Create room selector
        let roomSelectorLabel = document.createElement('label');
        roomSelectorLabel.setAttribute('for', 'room-selector');
        roomSelectorLabel.setAttribute('id', 'room-selector-label');
        roomSelectorLabel.innerText = 'Choose a room: ';
        let roomSelector = document.createElement('select');
        roomSelector.setAttribute('name', 'room-selector');
        roomSelector.setAttribute('id', 'room-selector');

        // Create room columns
        rooms.forEach((room, index) => {
            const column = document.createElement('div');
            column.className = 'column';
            column.setAttribute('data-room-id', room.id);
            column.innerHTML = `
                <div class="column-header">
                    ${room.name}
                    <button class="delete-room-btn" data-room-id="${room.id}">x</button>
                </div>`;
            column.addEventListener('dragover', handleDragOver);
            column.addEventListener('drop', handleDrop);
            scheduleContainer.appendChild(column);

            const option = document.createElement('option');
            option.value = room.id;
            option.textContent = room.name;
            roomSelector.appendChild(option);

            if (index === 0) {
                column.classList.add('active');
            }
        });

        displayControls.appendChild(roomSelectorLabel);
        displayControls.appendChild(roomSelector);

        // Add room selection handler
        roomSelector.addEventListener('change', (event) => {
            const selectedRoomId = event.target.value;
            document.querySelectorAll('.column').forEach(column => {
                column.classList.toggle(
                    'active',
                    column.getAttribute('data-room-id') === selectedRoomId
                );
            });
        });

        // Add events to room columns
        events.forEach(event => {
            const column = document.querySelector(`.column[data-room-id="${event.roomId}"]`);
            if (column) {
                const eventBlock = createEventBlock(event, 'time');
                column.appendChild(eventBlock);
            }
        });
    }

    function generateRoomBasedView() {
        scheduleContainer.innerHTML = '';
        let roomSelector = document.getElementById('room-selector');
        if (roomSelector) {
            roomSelector.remove();
            document.getElementById('room-selector-label').remove();
        }

        // Create rooms column
        let rowColumn = document.createElement('div');
        rowColumn.className = 'row-column';
        let rowColumnHeader = document.createElement('div');
        rowColumnHeader.className = 'column-header';
        rowColumnHeader.innerText = 'Room';
        rowColumn.appendChild(rowColumnHeader);

        // Add room labels
        rooms.forEach(room => {
            const rowLabel = document.createElement('div');
            rowLabel.className = 'row-label';
            rowLabel.textContent = room.name;
            rowColumn.appendChild(rowLabel);
        });

        scheduleContainer.appendChild(rowColumn);

        // Create time selector
        let timeSelectorLabel = document.createElement('label');
        timeSelectorLabel.setAttribute('for', 'time-selector');
        timeSelectorLabel.setAttribute('id', 'time-selector-label');
        timeSelectorLabel.innerText = 'Choose a starting time: ';
        let timeSelector = document.createElement('select');
        timeSelector.setAttribute('name', 'time-selector');
        timeSelector.setAttribute('id', 'time-selector');

        // Create time columns based on actual timeslots
        timeslots.forEach((slot, index) => {
            const column = document.createElement('div');
            column.className = 'column';
            column.setAttribute('data-time', slot.start.substring(0, 5));
            column.innerHTML = `<div class="column-header">${slot.start.substring(0, 5)}</div>`;
            column.addEventListener('dragover', handleDragOver);
            column.addEventListener('drop', handleDrop);
            scheduleContainer.appendChild(column);

            const option = document.createElement('option');
            option.value = slot.start.substring(0, 5);
            option.textContent = slot.start.substring(0, 5);
            timeSelector.appendChild(option);

            if (index === 0) {
                column.classList.add('active');
            }
        });

        displayControls.appendChild(timeSelectorLabel);
        displayControls.appendChild(timeSelector);

        // Add time selection handler
        timeSelector.addEventListener('change', (event) => {
            const selectedTime = event.target.value;
            document.querySelectorAll('.column').forEach(column => {
                column.classList.toggle(
                    'active',
                    column.getAttribute('data-time') === selectedTime
                );
            });
        });

        // Add events to time columns
        events.forEach(event => {
            const column = document.querySelector(`.column[data-time="${event.startTime.substring(0, 5)}"]`);
            if (column) {
                const eventBlock = createEventBlock(event, 'room');
                column.appendChild(eventBlock);
            }
        });
    }

    // Initialize view selector handler and initial view
    document.getElementById('view-selector').addEventListener('change', updateView);
    updateView();
});