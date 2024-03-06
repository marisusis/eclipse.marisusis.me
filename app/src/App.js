import logo from "./logo.svg";
import "./App.scss";
import Container from "react-bootstrap/Container";
import Row from "react-bootstrap/Row";
import Col from "react-bootstrap/Col";
import SignalGraph from "./components/SignalGraph";
import SignalPanel from "./components/SignalPanel";
import { useEffect, useState } from "react";

const chunkArray = (array, chunkSize) => {
  const result = [];
  for (let i = 0; i < array.length; i += chunkSize) {
    result.push(array.slice(i, i + chunkSize));
  }
  return result;
};

function App() {
  let [data, setData] = useState(null);
  let [waitingText, setWaitingText] = useState("Waiting for server");

  useEffect(() => {
    let timeout = null;
    let interval = setInterval(() => {
      let controller = new AbortController();
      const signal = controller.signal;

      timeout = setTimeout(() => {
        controller.abort();
      }, 500);

      fetch("/api/data/all", { signal: signal })
        .then((response) => {
          if (!response.ok) {
            return Promise.reject(response);
          }

          return response.json();
        })
        .then((data) => {
          setData(data);
        })
        .catch((error) => {
          console.error("Error fetching data:", error);
        });
    }, 1500);

    return () => {
      clearInterval(interval);
      if (timeout) clearTimeout(timeout);
    };
  }, []);

  let data_elems = <></>;

  if (data != null) {
    data_elems = data.data.map((node, index) => {
      let status = "offline";
      if (node.data != null) {
        if (node.data.flags.has_gps_fix) {
          status = "online";
        } else {
          status = "no-fix";
        }
      }

      return (
        <SignalPanel
          key={index}
          node_id={node.node_id}
          location={node.location}
          data={node.data}
          status={status}
        />
      );
    });
  }

  return (
    <Container fluid style={{ backgroundColor: "black" }}>
      <div className="panels">
        {data == null ? (
          <h1 id="waiting">Waiting for server...</h1>
        ) : (
          data_elems
        )}
      </div>
    </Container>
  );
}

export default App;
