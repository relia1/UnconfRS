let numOfTimeslots = 0;
let numOfRooms = 0;
let scheduleContainer = null;
let displayControls = null;

class EventElement {
    constructor(div) {
        this.element = div.element || div;
    }

    updatePosition(top, height) {
        this.element.style.top    = top;
        this.element.style.height = height;
    }

    moveTo(column) {
        column.appendChild(this.element);
    }

    updateAttributes(newData) {
        // Map properties to their corresponding data-attributes
        const attributeMap = {
            startTime:  'data-start-time',
            endTime:    'data-end-time',
            roomId:     'data-room-id',
            timeslotId: 'data-timeslot-id',
            sessionId: 'data-session-id',
            scheduleId: 'data-schedule-id',
        };

        // Only update attributes that are provided in newData
        Object.entries(attributeMap).forEach(([key, attr]) => {
            if (newData[key] !== undefined) {
                this.element.setAttribute(attr, newData[key]);
            }
        });

        if (newData.title !== undefined) {
            this.element.textContent = newData.title;
        }
    }

    getData() {
        return new EventData({...this.element.dataset, title: this.element.textContent});
    }
}

class EventData {
    constructor({roomId, timeslotId, sessionId, scheduleId, startTime, endTime, title}) {
        this.roomId     = Number(roomId);
        this.timeslotId = Number(timeslotId);
        this.sessionId = Number(sessionId);
        this.scheduleId = Number(scheduleId);
        this.startTime = startTime + ':00';
        this.endTime   = endTime + ':00';
        this.title      = title;
    }
}

document.addEventListener('DOMContentLoaded', function () {
    scheduleContainer = document.querySelector('.schedule-container');
    displayControls = document.getElementById('controls');
    let timeslots = window.APP.timeslots;
    let rooms = window.APP.rooms;
    let events = window.APP.events;

    numOfRooms = rooms.length;
    numOfTimeslots = timeslots.length;

    function createEventBlock(event, displayType) {
        const div     = document.createElement('div');
        div.className = 'event-block';

        const eventElement = new EventElement(div);
        eventElement.updateAttributes({
            startTime:  event.startTime.substring(0, 5),
            endTime:    event.endTime.substring(0, 5),
            roomId:     event.roomId,
            timeslotId: event.timeslotId,
            title:      event.title,
            sessionId: event.sessionId,
            scheduleId: event.scheduleId,
        });

        const {top, height} = displayType === 'time' ?
                              calculateEventPosition(event.startTime.substring(0, 5)) :
                              calculateRoomBasedEventPosition(event.roomId);

        eventElement.updatePosition(top, height);

        return div;
    }

    function calculatePosition(index, total) {
        const multiplier = (1 / (total + 1)) * 100;
        const top    = (((index + 1) * multiplier) + 1) + '%';
        const height = (multiplier - 1.5) + '%';
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
        return calculatePosition(rooms.findIndex(room => room.id === roomId), numOfRooms);
    }

    function updateView() {
        const view = document.getElementById('view-selector').value;
        view === 'time' ? generateTimeBasedView() : generateRoomBasedView();
    }

    function generateTimeBasedView() {
        scheduleContainer.innerHTML = '';

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

        // Create room columns
        rooms.forEach((room, index) => {
            const column = document.createElement('div');
            column.className = 'column';
            column.setAttribute('data-room-id', room.id);
            column.innerHTML = `
                <div class="column-header">
                    ${room.name}
                </div>`;
            scheduleContainer.appendChild(column);

            if (index === 0) {
                column.classList.add('active');
            }
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

        // Create time columns based on actual timeslots
        timeslots.forEach((slot, index) => {
            const column = document.createElement('div');
            column.className = 'column';
            column.setAttribute('data-time', slot.start.substring(0, 5));
            column.innerHTML = `<div class="column-header">${slot.start.substring(0, 5)}</div>`;
            scheduleContainer.appendChild(column);

            if (index === 0) {
                column.classList.add('active');
            }
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