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

    // Populate room and time selectors
    function populateSelectors() {
        const roomSelector = document.getElementById('room-selector');
        const timeSelector = document.getElementById('time-selector');

        // Populate room selector
        roomSelector.innerHTML = '';
        rooms.forEach(room => {
            const option       = document.createElement('option');
            option.value       = room.id;
            option.textContent = room.name;
            roomSelector.appendChild(option);
        });

        // Populate time selector
        timeSelector.innerHTML = '';
        timeslots.forEach(slot => {
            const option       = document.createElement('option');
            option.value       = slot.start.substring(0, 5);
            option.textContent = slot.start.substring(0, 5);
            timeSelector.appendChild(option);
        });
    }

    // Show/hide selectors based on view and screen size
    function updateSelectorVisibility() {
        const view                  = document.getElementById('view-selector').value;
        const roomSelectorContainer = document.getElementById('room-selector').parentElement;
        const timeSelectorContainer = document.getElementById('time-selector').parentElement;

        if (window.innerWidth <= 768) {
            if (view === 'time') {
                roomSelectorContainer.style.display = 'block';
                timeSelectorContainer.style.display = 'none';
            } else {
                roomSelectorContainer.style.display = 'none';
                timeSelectorContainer.style.display = 'block';
            }
        } else {
            roomSelectorContainer.style.display = 'none';
            timeSelectorContainer.style.display = 'none';
        }
    }

    // Handle room selector change
    function handleRoomChange() {
        const selectedRoomId = document.getElementById('room-selector').value;
        document.querySelectorAll('.column').forEach(column => {
            column.classList.remove('active');
        });
        const selectedColumn = document.querySelector(`.column[data-room-id="${selectedRoomId}"]`);
        if (selectedColumn) {
            selectedColumn.classList.add('active');
        }
    }

    // Handle time selector change
    function handleTimeChange() {
        const selectedTime = document.getElementById('time-selector').value;
        document.querySelectorAll('.column').forEach(column => {
            column.classList.remove('active');
        });
        const selectedColumn = document.querySelector(`.column[data-time="${selectedTime}"]`);
        if (selectedColumn) {
            selectedColumn.classList.add('active');
        }
    }

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
        updateSelectorVisibility();

        // Initialize Bootstrap tooltips after view is updated
        setTimeout(() => {
            // Dispose existing tooltips first
            const existingTooltips = document.querySelectorAll('[data-bs-toggle="tooltip"]');
            existingTooltips.forEach(element => {
                const tooltip = bootstrap.Tooltip.getInstance(element);
                if (tooltip) {
                    tooltip.dispose();
                }
            });

            // Initialize new tooltips
            const tooltipTriggerList = document.querySelectorAll(
                '.row-label[data-bs-toggle="tooltip"]');
            console.log('Initializing tooltips for', tooltipTriggerList.length, 'elements');
            tooltipTriggerList.forEach(element => {
                new bootstrap.Tooltip(element);
            });
        }, 100);
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
                // Set the room selector to match the active room
                document.getElementById('room-selector').value = room.id;
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

            const roomNameSpan       = document.createElement('span');
            roomNameSpan.className   = 'room-name';
            roomNameSpan.textContent = room.name;
            rowLabel.appendChild(roomNameSpan);

            rowLabel.setAttribute('data-bs-toggle', 'tooltip');
            rowLabel.setAttribute('data-bs-placement', 'right');
            rowLabel.setAttribute('data-bs-title', room.name);
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
                // Set the time selector to match the active timeslot
                document.getElementById('time-selector').value = slot.start.substring(0, 5);
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

    // Initialize selectors and event listeners
    populateSelectors();
    document.getElementById('view-selector').addEventListener('change', updateView);
    document.getElementById('room-selector').addEventListener('change', handleRoomChange);
    document.getElementById('time-selector').addEventListener('change', handleTimeChange);
    window.addEventListener('resize', updateSelectorVisibility);
    
    updateView();
});
