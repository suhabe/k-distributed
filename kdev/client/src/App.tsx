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

type Job = {
    id: number,
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
    program: string,
    spec: string,
    jobs: Job[]
};

class App extends React.Component<AppProps, AppState> {
    constructor(props:any) {
        super(props);
        this.state = {
            program: "",
            spec: "",
            jobs: [] as Job[],
        };

        this.handleSubmit = this.handleSubmit.bind(this);
        this.handleProgramChange = this.handleProgramChange.bind(this);
        this.handleSpecChange = this.handleSpecChange.bind(this);
    }

    handleProgramChange(e:any) {
        this.setState({program: e.target.value});
    }

    handleSpecChange(e:any) {
        this.setState({spec: e.target.value});
    }

    handleReload() {

        console.log(this.state.jobs);

        return fetch('/reload', {
            method: 'POST',
            headers: {
                'Accept': 'application/json',
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({job_ids: this.state.jobs.map(job => job.id)})
        })  .then((response) => response.json())
            .then((json) => this.setState({...this.state, jobs: json.jobs as Job[]}))
            .catch(function(error) { console.log("error")});
    }

    handleSubmit(event:any) {
        event.preventDefault();

        return fetch('/prove', {
            method: 'POST',
            headers: {
                'Accept': 'application/json',
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({program: this.state.program, spec: this.state.spec})
        })  .then((response) => response.json())
            .then((json) => {
                let jobs = json.jobs as Job[];
                console.log(jobs);
                this.setState({...this.state, jobs: jobs });
            }).catch(function(error) { console.log(error)});
    }

  render() {
      let job_rows = this.state.jobs.map(job => {
          return <tr>
              <td>{job.spec_name}</td>
              <td>{job.request_dt}</td>
              <td>{job.processed_dt}</td>
              <td>{job.completed_dt}</td>
              <td>{job.processing_mins}</td>
              <td>{job.status_code}</td>
              <td>{job.result}</td>
              <td><a href="{job.out_url}">stdout</a></td>
              <td><a href="{job.err_url}">stderr</a></td>
          </tr>
      });

      return (
        <Container fluid>
            <Form onSubmit={this.handleSubmit}>
                <Row>
                    <Col>
                        <Form.Group controlId="exampleForm.ControlTextarea1">
                            <Form.Label>Program</Form.Label>
                            <Form.Control as="textarea" rows="20" onChange={this.handleProgramChange} />
                        </Form.Group>
                    </Col>
                    <Col>
                        <Form.Group controlId="exampleForm.ControlTextarea1">
                            <Form.Label>Specification</Form.Label>
                            <Form.Control as="textarea" rows="20" onChange={this.handleSpecChange}/>
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
                                <th>Spec</th>
                                <th>Requested</th>
                                <th>Processed</th>
                                <th>Completed</th>
                                <th>Time</th>
                                <th>Status Code</th>
                                <th>Result</th>
                                <th>Output</th>
                                <th>Error</th>
                            </tr>
                        </thead>
                        <tbody>
                        { job_rows }
                        </tbody>
                    </Table>
                </Col>
            </Row>
            <Row>
                <Col>
                    <Button variant="primary" type="submit" onClick={() => this.handleReload()}>
                        Refresh
                    </Button>
                </Col>
            </Row>
        </Container>
  );
  }
}

export default App;
