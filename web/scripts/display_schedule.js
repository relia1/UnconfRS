document.addEventListener('DOMContentLoaded', function () {
    if (localStorage.getItem('admin') === 'true') {
        document.body.classList.add('admin');
    }

    let numOfHalfHourSegments = 0;
    let numOfRooms = 0;
    const scheduleContainer = document.querySelector('.schedule-container');
    const displayControls = document.getElementById('controls');

    const rooms = [
        {% if let Some(vec_rooms) = rooms %}
            {% for room in vec_rooms %}
                {
                    id: Number({{ room.id.unwrap() }}),
                    name: "{{ room.name }}",
                    available_spots: Number({{ room.available_spots }})
                },
            {% endfor %}
        {% endif %}
    ];

    numOfRooms = rooms.length;
    const events = [
        {% for topic_event in events %}
            {
                roomId: Number({{ topic_event.room_id }}),
                startTime: "{{ topic_event.start_time }}",
                endTime: "{{ topic_event.end_time }}",
                timeslotId: Number({{ topic_event.timeslot_id }}),
                title: "{{ topic_event.title }}",
                topicId: Number({{ topic_event.topic_id }}),
                speakerId: Number({{ topic_event.speaker_id }}),
                scheduleId: Number({{ topic_event.schedule_id }})
            },
        {% endfor %}
    ];

    let viewSelectorValue = document.getElementById('view-selector').value;


    /* Takes an event and creates an event div element with the
    // position within the room column applied to it
    */
    function createEventBlock(event, displayType) {
        const eventBlock = document.createElement('div');
        eventBlock.className = 'event-block';
        eventBlock.textContent = event.title;
        eventBlock.draggable = true;
        eventBlock.setAttribute('data-start-time', event.startTime);
        eventBlock.setAttribute('data-end-time', event.endTime);
        eventBlock.setAttribute('data-timeslot-id', event.timeslotId);
        eventBlock.setAttribute('data-topic-id', event.topicId);
        eventBlock.setAttribute('data-speaker-id', event.speakerId);
        eventBlock.setAttribute('data-schedule-id', event.scheduleId);
        eventBlock.setAttribute('data-room-id', event.roomId);
        eventBlock.addEventListener('dragstart', handleDragStart);

        if (displayType.localeCompare('time') === 0) {
            const {top, height} = calculateEventPosition(event.startTime, event.endTime);
            eventBlock.style.top = top;
            eventBlock.style.height = height;
        } else {
            const {top, height} = calculateRoomBasedEventPosition(event.roomId);
            eventBlock.style.top = top;
            eventBlock.style.height = height;
        }

        return eventBlock;
    }

    /* Calculate the position within the room column for the event
    // to be shown at
    */
    function calculateEventPosition(startTime, endTime) {
        /* Multiplier used for determining position and height of event
        // 1 is added due to the header taking up the first row of the
        // room column
        */
        const timeHeightMultiplier = (1 / (numOfHalfHourSegments + 1)) * 100;
        const startMinutes = timeToMinutes(startTime);
        const endMinutes = timeToMinutes(endTime);
        const duration = endMinutes - startMinutes;
        /* 8:00 AM in minutes */
        const scheduleStart = 8 * 60;
        /* 6:00 PM in minutes */
        const scheduleEnd = 18 * 60;
        /* Time in minutes from the start of the schedule
        // to the end in minutes
        */
        const scheduleLength = scheduleEnd - scheduleStart;
        /* The amount of time that has elapsed since the start
        // of the schedule
        */
        const timeSinceScheduleStart = startMinutes - scheduleStart;
        /* Number of segments into the rooms column */
        const numOf30MinSegmentsSinceStart = ((timeSinceScheduleStart) / 30) + 1;
        /*
        console.log('start minutes ' + `${startMinutes}`);
        console.log('end minutes ' + `${endMinutes}`);
        console.log('duration ' + `${duration}`);
        console.log('schedule start ' + `${scheduleStart}`);
        console.log('schedule end ' + `${scheduleEnd}`);
        console.log('schedule length ' + `${scheduleLength}\n\n`);
        console.log('num 30 min segs ' + `${numOf30MinSegmentsSinceStart}`);
        console.log('time height multiplier ' + `${timeHeightMultiplier}`);
        */
        /* Using the number of segments into the rooms column and
        // the multiplier gives us the starting time location of
        // the event
        */
        const top = (Math.ceil((numOf30MinSegmentsSinceStart * timeHeightMultiplier) * 100) / 100) + '%';
        /* Using the number of segments for the duration of the event
        // we determine how long the event lasts
        */
        const height = (Math.floor(((duration / 30.0) * timeHeightMultiplier) * 10) / 10) + '%';

        /* console.log(`top: ${top} height: ${height}`); */
        return {top, height};
    }

    /* Calculate the position within the room column for the event
    // to be shown at
    */
    function calculateRoomBasedEventPosition(roomId) {
        /* Multiplier used for determining position and height of event
        // 1 is added due to the header taking up the first row of the
        // room column
        */
        const roomHeightMultiplier = (1 / (numOfRooms + 1)) * 100;
        /* Using the room ID determine the starting location of the event */
        const top = ((roomId) * roomHeightMultiplier) + '%';
        /* Since rooms don't have a length the height multiplier
        // is the height of the event
        */
        const height = roomHeightMultiplier + '%';

        /* console.log(`top: ${top} height: ${height}`); */
        return {top, height};
    }


    /* Given time in the format of 'hours:minutes' return back
    // time only in minutes
    */
    function timeToMinutes(time) {
        const [hours, minutes] = time.split(':').map(Number);
        return hours * 60 + minutes;
    }

    /* Given time in minutes return back time in 'hours:minutes' */
    function minutesToTime(minutes) {
        const hours = Math.floor(minutes / 60);
        const mins = minutes % 60;
        return `${hours.toString().padStart(2, '0')}:${mins.toString().padStart(2, '0')}`;
    }

    /* Drag and drop event handlers */
    let draggedEvent = null;

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
        (event.preventDefault());
        (event.dataTransfer.dropEffect = "move");
    }

    async function handleDrop(event) {
        (event.preventDefault());
        if (draggedEvent) {
            const roomColumn = (event.target.closest('.column'));
            const rect = roomColumn.getBoundingClientRect();
            const y = (event.clientY - rect.top);
            /* 10 hours in minutes */
            const scheduleLength = 10 * 60;
            const minutesFromStart = (Math.round((y / rect.height) * ((scheduleLength / 30))) - 1) * 30;
            /* 8:00 AM + mintues from start */
            const newStartMinutes = 8 * 60 + minutesFromStart;
            const newStartTime = minutesToTime(newStartMinutes);

            /* Calculate event duration */
            const startTime = draggedEvent.getAttribute('data-start-time');
            const endTime = draggedEvent.getAttribute('data-end-time');
            const duration = timeToMinutes(endTime) - timeToMinutes(startTime);

            /* Calculate new end time */
            const newEndMinutes = newStartMinutes + duration;
            const newEndTime = minutesToTime(newEndMinutes);

            const roomId = roomColumn.getAttribute('data-room-id');
            /* Update event block */
            draggedEvent.setAttribute('data-start-time', newStartTime);
            draggedEvent.setAttribute('data-end-time', newEndTime);
            draggedEvent.setAttribute('data-room-id', roomId);

            const {top, height} = calculateEventPosition(newStartTime, newEndTime);
            draggedEvent.style.top = top;
            draggedEvent.style.height = height;

            roomColumn.appendChild(draggedEvent);
            const timeslotId = draggedEvent.getAttribute('data-timeslot-id');
            const topicId = draggedEvent.getAttribute('data-topic-id');
            const speakerId = draggedEvent.getAttribute('data-speaker-id');
            const scheduleId = draggedEvent.getAttribute('data-schedule-id');
            /* Update moved event */
            const timeslot = {
                id: Number(timeslotId),
                start_time: newStartTime,
                end_time: newEndTime,
                speaker_id: Number(speakerId),
                topic_id: Number(topicId),
                room_id: Number(roomId),
                schedule_id: Number(scheduleId),
            };

            let eventIndex = events.findIndex(obj => obj.timeslotId === timeslot.id);
            events[eventIndex].startTime = timeslot.start_time;
            events[eventIndex].endTime = timeslot.end_time;
            events[eventIndex].roomId = timeslot.room_id;
            try {
                const response = await fetch(`api/v1/timeslots/${timeslot.id}`, {
                    method: 'PUT',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify(timeslot),
                });

                if (response.status === 401) {
                    alert('You do not have permission to move events');
                    throw new Error(`HTTP error! status: ${response.status}`);
                }

                if (!response.ok) {
                    alert('There was an error populating the schedule. Please try again.');
                    throw new Error(`HTTP error! status: ${response.status}`);
                }
            } catch (error) {
                console.error('Error updating the schedule:', error);
                alert('There was an error updating the schedule.');
            }

            draggedEvent.setAttribute('data-timeslot-id', events[eventIndex].timeslotId);
            draggedEvent = null;
        }
    }

    document.getElementById('populate-schedule').addEventListener('click', async (e) => {
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

    document.getElementById('clear-schedule').addEventListener('click', async (e) => {
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

    document.getElementById('add-room').addEventListener('click', async (e) => {
        const popup = document.getElementById('popup');
        const overlay = document.getElementById('overlay');
        popup.style.display = 'block';
        overlay.style.display = 'block';
        document.querySelector('#cancelButton').addEventListener('click', closePopup);
        document.querySelector('#overlay').addEventListener('click', closePopup);
    });

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

    document.getElementById('submit-btn').addEventListener('click', async (e) => {
        e.preventDefault();
        const createRoomsForm = {
            rooms: [{
                name: document.getElementById('room-name').value,
                location: document.getElementById('room-location').value,
                available_spots: Number(20)
            }]
        }

        try {
            const response = await fetch('/api/v1/rooms/add', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(createRoomsForm)
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

    function updateView() {
        const view = document.getElementById('view-selector').value;
        const container = document.querySelector('.schedule-container');
        view === 'time' ? generateTimeBasedView() : generateRoomBasedView();
    }

    function generateTimeBasedView() {
        console.log('time based');
        /* Clear out schedule and generate the time based view */
        scheduleContainer.innerHTML = '';
        let timeSelector = document.getElementById('time-selector');
        if (timeSelector) {
            timeSelector.remove();
            document.getElementById('time-selector-label').remove();
        }
        numOfHalfHourSegments = 0;

        /* Create time slots */
        let rowColumn = document.createElement('div');
        rowColumn.className = 'row-column';
        let rowColumnHeader = document.createElement('div');
        rowColumnHeader.className = 'column-header';
        rowColumnHeader.innerText = 'Time';
        rowColumn.appendChild(rowColumnHeader);
        scheduleContainer.appendChild(rowColumn);

        rowColumn = document.querySelector('.row-column');
        for (let hour = 8; hour < 18; hour++) {
            for (let minute = 0; minute < 60; minute += 30) {
                numOfHalfHourSegments += 1;
                const rowLabel = document.createElement('div');
                rowLabel.className = 'row-label';
                rowLabel.textContent = `${hour.toString().padStart(2, '0')}:${minute.toString().padStart(2, '0')}`;
                rowColumn.appendChild(rowLabel);
            }
        }

        let roomSelectorLabel = document.createElement('label');
        roomSelectorLabel.setAttribute('for', 'room-selector');
        roomSelectorLabel.setAttribute('id', 'room-selector-label');
        roomSelectorLabel.innerText = 'Choose a room: ';
        let roomSelector = document.createElement('select');
        roomSelector.setAttribute('name', 'room-selector');
        roomSelector.setAttribute('id', 'room-selector');
        displayControls.appendChild(roomSelectorLabel);

        /* Create room columns */
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
        displayControls.appendChild(roomSelector);

        roomSelector.addEventListener('change', (event) => {
            const selectRoomId = event.target.value;
            document.querySelectorAll('.column').forEach(column => {
                if (column.getAttribute('data-room-id') === selectRoomId) {
                    column.classList.add('active');
                } else {
                    column.classList.remove('active');
                }
            });
        });


        /* Add events to room columns */
        events.forEach(event => {
            const column = document.querySelector(`.column[data-room-id="${event.roomId}"]`);
            const eventBlock = createEventBlock(event, 'time');
            column.appendChild(eventBlock);
        });

    }

    function generateRoomBasedView() {
        console.log('room based');
        /* Clear out schedule and generate the room based view */
        scheduleContainer.innerHTML = '';
        let roomSelector = document.getElementById('room-selector');
        if (roomSelector) {
            roomSelector.remove();
            document.getElementById('room-selector-label').remove();
        }

        numOfHalfHourSegments = 0;

        /* Create time slots */
        let rowColumn = document.createElement('div');
        rowColumn.className = 'row-column';
        let rowColumnHeader = document.createElement('div');
        rowColumnHeader.className = 'column-header';
        rowColumnHeader.innerText = 'Room';
        rowColumn.appendChild(rowColumnHeader);
        scheduleContainer.appendChild(rowColumn);
        rowColumn = document.querySelector('.row-column');

        rooms.forEach((room, index) => {
            const rowLabel = document.createElement('div');
            rowLabel.className = 'row-label';
            rowLabel.innerHtml = `
                ${room.name}
                <button class="delete-room-btn" data-room-id="${room.id}">x</button>
            `;
            rowColumn.appendChild(rowLabel);
        });

        let timeSelectorLabel = document.createElement('label');
        timeSelectorLabel.setAttribute('for', 'time-selector');
        timeSelectorLabel.setAttribute('id', 'time-selector-label');
        timeSelectorLabel.innerText = 'Choose a starting time: ';
        let timeSelector = document.createElement('select');
        timeSelector.setAttribute('name', 'time-selector');
        timeSelector.setAttribute('id', 'time-selector');
        displayControls.appendChild(timeSelectorLabel);


        for (let hour = 8; hour < 18; hour++) {
            numOfHalfHourSegments += 1;
            for (let minute = 0; minute < 60; minute += 30) {
                let column = document.createElement('div');
                column.className = 'column';
                let eventHour = hour < 10 ? `0${hour}` : `${hour}`;
                let eventMinute = minute === 0 ? '00' : '30';
                column.setAttribute('data-time', `${eventHour}:${eventMinute}`);
                column.innerHTML = `<div class="column-header">${eventHour}:${eventMinute}</div>`;
                column.addEventListener('dragover', handleDragOver);
                column.addEventListener('drop', handleDrop);
                scheduleContainer.appendChild(column);

                let option = document.createElement('option');
                option.value = `${eventHour}:${eventMinute}`;
                option.textContent = `${eventHour}:${eventMinute}`;
                timeSelector.appendChild(option);

                if (hour === 8 && minute === 0) {
                    column.classList.add('active');
                }
            }
        }

        timeSelector.addEventListener('change', (event) => {
            const selectStartTime = event.target.value;
            document.querySelectorAll('.column').forEach(column => {
                let startTime = event.currentTarget.value;
                console.log(`actual start time ${column.getAttribute('data-time')}`);
                console.log(`start time ${startTime}`);
                if (column.getAttribute('data-time').localeCompare(startTime) === 0) {
                    column.classList.add('active');
                } else {
                    column.classList.remove('active');
                }
            });
        });

        displayControls.appendChild(timeSelector);

        /* Add events to room columns */
        events.forEach(event => {
            let column = document.querySelector(`.column[data-time="${event.startTime.substring(0, event.startTime.length - 3)}"]`);
            const eventBlock = createEventBlock(event, 'room');
            column.appendChild(eventBlock);
        });

    }

    document.getElementById('view-selector').addEventListener('change', updateView);
    /* initial view */
    updateView();
});