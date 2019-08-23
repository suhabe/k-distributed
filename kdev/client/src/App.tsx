import React from 'react';
import logo from './logo.svg';
import './App.css';
import Container from 'react-bootstrap/Container';
import Row from 'react-bootstrap/Row';
import Col from 'react-bootstrap/Col';
import Form from "react-bootstrap/Form";
import ButtonToolbar from "react-bootstrap/ButtonToolbar";
import Button from "react-bootstrap/Button";
import Table from "react-bootstrap/Table";

type ProveResult = {
    completed_dt: string,
    request_dt: string,
    processed_dt: string,
    processing_secs: string,
    processing_mins: string,
    benchmark_name: string,
    spec_name: string,
    status_code: string,
    out_url: string,
    err_url: string,
    result: string,
    result_color: string,
}

type AppProps = {  };
type AppState = {
    proveResult: ProveResult
};



class App extends React.Component<AppProps, AppState> {
    constructor(props:any) {
        super(props);
        this.state = {proveResult: {
                completed_dt: "",
                request_dt: "",
                processed_dt: "",
                processing_secs: "",
                processing_mins: "",
                benchmark_name: "",
                spec_name: "",
                status_code: "",
                out_url: "",
                err_url: "",
                result: "",
                result_color: "",
            }
        };
        this.handleSubmit = this.handleSubmit.bind(this);
    }

    handleSubmit(event:any) {
        event.preventDefault();

        console.log("handle request ");
        //alert('Your favorite flavor is: ' + this.state.value);
        //event.preventDefault();
        return fetch('/prove', {
            method: 'POST',
            headers: {
                'Accept': 'application/json',
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({program: "fn {}", spec: '<k> ... </k>'})
        })  .then((response) => response.json())
            .then((json) => this.setState({ proveResult: json.row}))
            .catch(function(error) { console.log("error")});
    }

  render() {
      return (
        <Container fluid>
            <Form onSubmit={this.handleSubmit}>
                <Row>
                    <Col>
                        <Form.Group controlId="exampleForm.ControlTextarea1">
                            <Form.Label>Program</Form.Label>
                            <Form.Control as="textarea" rows="20" />
                        </Form.Group>
                    </Col>
                    <Col>
                        <Form.Group controlId="exampleForm.ControlTextarea1">
                            <Form.Label>Specification</Form.Label>
                            <Form.Control as="textarea" rows="20" />
                        </Form.Group>
                    </Col>
                </Row>
                <Row>
                    <Col>
                        <Button variant="primary" type="submit">
                            Prove
                        </Button>
                    </Col>
                </Row>
            </Form>
            <Row>
                <Col>
                    <Table bordered>
                        <thead>
                            <tr>
                                <th>Time</th>
                                <th>Status Code</th>
                                <th>Result</th>
                                <th>Output</th>
                                <th>Error</th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr>
                                <td>{this.state.proveResult.processing_mins}</td>
                                <td>{this.state.proveResult.status_code}</td>
                                <td>{this.state.proveResult.result}</td>
                                <td><a href="{{out_url}}">stdout</a></td>
                                <td><a href="{{err_url}}">stderr</a></td>
                            </tr>
                        </tbody>
                    </Table>
                </Col>
            </Row>
        </Container>
  );
  }
}

export default App;
