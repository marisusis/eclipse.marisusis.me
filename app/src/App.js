import logo from "./logo.svg";
import "./App.scss";
import Container from "react-bootstrap/Container";
import Row from "react-bootstrap/Row";
import Col from "react-bootstrap/Col";
import SignalGraph from "./components/SignalGraph";
import SignalPanel from "./components/SignalPanel";

const chunkArray = (array, chunkSize) => {
  const result = [];
  for (let i = 0; i < array.length; i += chunkSize) {
    result.push(array.slice(i, i + chunkSize));
  }
  return result;
};

function App() {
  let graphs = [
    {
      node_id: "ET0002",
      endpoint: "/api/data/et0002",
      location: "Village House 3, 232D",
    },
    {
      node_id: "ET1001",
      endpoint: "/api/data/et1001",
    },
    {
      node_id: "ET1002",
      endpoint: "/api/data/et1002",
      location: "Glennan Building, CWRU, OH",
    },
    {
      node_id: "ET1003",
      endpoint: "/api/data/et1003",
      location: "Glennan Building, CWRU, OH",
    },
    {
      node_id: "ET1004",
      endpoint: "/api/data/et1004",
    },
    {
      node_id: "ET1005",
      endpoint: "/api/data/et1005",
    },
    {
      node_id: "ET1006",
      endpoint: "/api/data/et1006",
    },
    {
      node_id: "ET1007",
      endpoint: "/api/data/et1007",
    },
    {
      node_id: "ET1008",
      endpoint: "/api/data/et1008",
    },
    {
      node_id: "ET1009",
      endpoint: "/api/data/et1009",
    },
    {
      node_id: "ET1010",
      endpoint: "/api/data/et1010",
    },
    {
      node_id: "ET1011",
      endpoint: "/api/data/et1011",
    },
    {
      node_id: "ET1012",
      endpoint: "/api/data/et1012",
    },
    {
      node_id: "ET1013",
      endpoint: "/api/data/et1013",
    },
    {
      node_id: "ET1014",
      endpoint: "/api/data/et1014",
    },
    {
      node_id: "ET1015",
      endpoint: "/api/data/et1015",
    },
    {
      node_id: "ET1016",
      endpoint: "/api/data/et1016",
    },
  ];

  const chunkedGraphs = chunkArray(graphs, 3);

  // let graphs = [
  //   {
  //     node_id: "ET1003",
  //     endpoint: "http://localhost:8080/data/et1003",
  //   },
  //   {
  //     node_id: "ET1002",
  //     endpoint: "http://localhost:8080/data/et1002",
  //   },
  //   {
  //     node_id: "ET1001",
  //     endpoint: "http://localhost:8080/data/et1001",
  //   },
  // ];

  // {chunkedGraphs.map((graphRow, rowIndex) => (
  //   <Row key={rowIndex}>
  //     {graphRow.map((graph, index) => (
  //       <Col key={index} xs={12} xxl={4} lg={6}>
  //         {/* Adjust column sizes as needed */}
  //         <SignalPanel config={graph} />
  //       </Col>
  //     ))}
  //   </Row>
  // ))}

  return (
    <Container fluid style={{ backgroundColor: "black" }}>
      {/* <Row>
        <h1 style={{ textAlign: "center", color: "white" }}>Signal Viewer</h1>
      </Row> */}
      {/* <Row>
        {graphs.map((graph, index) => (
          <Col key={index} xs={12} xxl={4} lg={6}>
            <SignalPanel config={graph} />
          </Col>
        ))}
      </Row> */}

      <div className="panels">
        {graphs.map((graph, index) => (
          // <div key={index} xs={12} xxl={4} lg={6}>
          <SignalPanel config={graph} />
          // </div>
        ))}
      </div>
    </Container>
  );
}

export default App;
