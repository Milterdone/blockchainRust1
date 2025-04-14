# Cryptocurrency Info Aggregator in Rust

This project is a Rust-based service that aggregates cryptocurrency information by calling the CoinMarketCap Info API. Instead of processing and reformatting the API response, the service forwards the raw JSON returned by CoinMarketCap to the client. The application also implements JWT-based authentication, error handling, and a WebSocket endpoint for real-time updates.

## Features

- **JWT Authentication:** Secure endpoints with a login endpoint that returns a JWT token.
- **Coin Info Endpoint:** Fetch and display raw JSON data from the CoinMarketCap Info API for a given cryptocurrency symbol.
- **WebSocket Endpoint:** Real-time updates through a WebSocket connection that supports heartbeat pings and simple echo functionality.
- **Simple Frontend:** A basic HTML form to facilitate quick testing of the API endpoints.
- **API Key Management:** Loads the CoinMarketCap API key from a `.env` file using the `dotenv` crate.

## Technology Stack

- **Backend:** Rust with [Actix Web](https://actix.rs/) and [Actix Web Actors](https://github.com/actix/actix-web-actors)
- **HTTP Client:** [Reqwest](https://docs.rs/reqwest)
- **Authentication:** [jsonwebtoken](https://docs.rs/jsonwebtoken)
- **Environment Variables:** [dotenv](https://docs.rs/dotenv)
- **WebSocket:** Actix Web Actors for real-time communication

## Requirements

- [Rust](https://www.rust-lang.org/tools/install) and Cargo installed
- A valid CoinMarketCap API key
- (Optional) A WebSocket client such as [wscat](https://www.npmjs.com/package/wscat) for testing the `/ws` endpoint

## Installation

1. **Clone the Repository:**
   ```bash
   git clone https://github.com/YourUsername/crypto_info_aggregator.git
   cd crypto_info_aggregator
   ```

2. **Set Up Environment Variables:**

   Create a `.env` file in the project root directory and add your API key:
   ```env
   COINMARKETCAP_API_KEY=YOUR_COINMARKETCAP_API_KEY
   ```

3. **Build the Project:**
   ```bash
   cargo build
   ```

4. **Run the Project:**
   ```bash
   cargo run
   ```
   
   The server will start at [http://127.0.0.1:8080](http://127.0.0.1:8080).

## Endpoints

### 1. Login

- **URL:** `POST /login`
- **Description:** Accepts a JSON payload with a username and password. For valid credentials (username: `admin`, password: `password`), it returns a JWT token.
- **Example cURL Command (Windows CMD):**
  ```cmd
  curl -X POST -H "Content-Type: application/json" -d "{\"username\": \"admin\", \"password\": \"password\"}" http://127.0.0.1:8080/login
  ```

### 2. Fetch Coin Info

- **URL:** `GET /news?symbol=BTC&token=YOUR_JWT_TOKEN`
- **Description:** Validates the provided JWT token and fetches raw JSON data from the CoinMarketCap Info API for the provided cryptocurrency symbol.
- **Example cURL Command (Windows CMD):**
  ```cmd
  curl -X GET "http://127.0.0.1:8080/news?symbol=BTC&token=YOUR_JWT_TOKEN"
  ```

### 3. WebSocket Endpoint

- **URL:** `ws://127.0.0.1:8080/ws`
- **Description:** A WebSocket endpoint that sends periodic heartbeat pings and echoes any messages sent by the client.
- **Testing:**
  Use a WebSocket client such as [wscat](https://www.npmjs.com/package/wscat):
  ```cmd
  wscat -c ws://127.0.0.1:8080/ws
  ```

### 4. HTML Interface

- **URL:** [http://127.0.0.1:8080/](http://127.0.0.1:8080/)
- **Description:** A simple HTML page is provided to test the `/news` endpoint via a form that accepts a cryptocurrency symbol and JWT token.

## Project Structure

```
crypto_info_aggregator/
├── Cargo.toml
├── .env                # Contains your COINMARKETCAP_API_KEY
└── src/
    └── main.rs        # Main application code for API endpoints and WebSocket server
```

## Bonus & Future Improvements

- **Advanced Features:** Enhance error handling, logging, and add more complex API integrations.
- **Frontend Improvements:** Consider building a richer frontend using frameworks like Yew for a more interactive experience.
- **Robust Authentication:** Expand authentication and authorization mechanisms for multiple users.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
