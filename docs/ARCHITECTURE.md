# Architecture

[<- Back to Main README](../README.md) | [Development](DEVELOPMENT.md)

## System Overview

UnconfRS is a web application using the Rust backend framework Axum and mostly plain HTML and JS on the frontend.
The application is mostly rendered server side, but has some interactive JS rendering.

## Technology Stack

### Backend Technologies

#### Rust

Programming language for the backend, provides:

- Memory safety and performance
- Strong typing
- Fearless concurrency

#### Axum

Web framework for handling requests and routing:

- Uses Tokio async runtime
- Declaratively parse requests using extractors
- Generate responses with minimal boilerplate
- Intergration with tower ecosystem

#### SqlX

Async SQL toolkit for Rust

- Statically checked SQL queries
- Connection pooling
- Migration management
- Async database operations

#### Askama

Template engine for server side HTML generation

- Statically checked templates
- Rust syntax integration in templates

### Frontend Technologies

#### HTML

Providing the page structure

- From server side rendered templates

#### CSS

Styling and responsiveness

- Custom styling for application

#### JavaScript

Client side interactivity

- Form submissions
- Dynamic table interactions
- Drag and drop for scheduling

#### DataTables

jQuery plugin for enhanced table functionality

- Sorting and filtering
- Pagination
- Search
- Responsive table

#### IDB-Keyval

Small promise based keyval store

- Persistent data storage in browser

### Database

#### PostgreSQL

Relational database management system

- Performant
- ACID compliant

## System Architecture

### Request Flow

### Data Flow

#### Server Side

#### Client Side

### Design Patterns

SSR/MVC
