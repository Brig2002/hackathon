# Initial commit

# D-Voting-App Project Proposal

## Overview

DappVotes is a decentralized application (dApp) designed to facilitate secure and transparent poll creation and voting using blockchain technology. It allows users to create polls, manage them, and participate in voting, ensuring immutability and trustworthiness of the results.

## Problem Statement

Traditional polling systems often suffer from issues such as lack of transparency, susceptibility to tampering, and limited accessibility. These issues undermine the trust in the polling process and can lead to disputes over the results.

## Objectives

- Develop a decentralized application for creating and managing polls.
- Ensure the transparency and immutability of the poll data using blockchain technology.
- Provide a secure and user-friendly interface for users to create polls and vote.

## Smart Contract Description

The DappVotes smart contract manages the lifecycle of polls, including their creation, retrieval, and updating. It ensures that all poll data is securely stored on the blockchain and can be accessed by users in a transparent manner.

### Key Functions

- createPoll: Allows the creation of a new poll with specified details such as title, description, image, start time, and end time.
- getPolls: Retrieves the list of all created polls.
- updatePoll: Allows updating the details of an existing poll.

## Test Files Description

The provided test files ensure the correct functionality of the DappVotes smart contract. They include tests for successful poll creation and updating, verifying that the contract behaves as expected.

### Sample Test Code

Here is a snippet from the test file:const { expectRevert } = require('@openzeppelin/test-helpers');
const { expect } = require('chai');
const { SigningCosmWasmClient } = require('secretjs');
const { Secp256k1Pen, encodeSecp256k1Pubkey } = require('secretjs/crypto');
const { EnigmaUtils, Secp256k1Pen } = require('secretjs');

describe('DappVotes Contract', function () {
    let client, contract, deployer, contestant1, contestant2, voter1, voter2, voter3;
    let pollId, contestantId;
    const name1 = 'Contestant 1';
    const name2 = 'Contestant 2';
    const avatar1 = 'https://avatar1.png';
    const avatar2 = 'https://avatar2.png';
    const description = 'Lorem Ipsum';
    const title = 'Republican Primary Election';
    const image = 'https://image.png';
    const starts = Math.floor(Date.now() / 1000) - 10 * 60;
    const ends = Math.floor(Date.now() / 1000) + 10 * 60;

    before(async function () {
        const mnemonic = "<YOUR_MNEMONIC>"; // Replace with your mnemonic
        const pen = await Secp256k1Pen.fromMnemonic(mnemonic);
        const pubkey = encodeSecp256k1Pubkey(pen.pubkey);

        const signingClient = new SigningCosmWasmClient(
            "https://api.secret.network",
            pubkey,
            (signBytes) => pen.sign(signBytes),
            EnigmaUtils.makeSignBytes
        );

        client = signingClient;
        contract = "<CONTRACT_ADDRESS>"; // Replace with your contract address

        deployer = "<DEPLOYER_ADDRESS>"; // Replace with your deployer address
        contestant1 = "<CONTESTANT1_ADDRESS>"; // Replace with contestant 1 address
        contestant2 = "<CONTESTANT2_ADDRESS>"; // Replace with contestant 2 address
        voter1 = "<VOTER1_ADDRESS>"; // Replace with voter 1 address
        voter2 = "<VOTER2_ADDRESS>"; // Replace with voter 2 address
        voter3 = "<VOTER3_ADDRESS>"; // Replace with voter 3 address
    });

    describe('Poll Management', function () {
        beforeEach(async function () {
            const handleMsg = {
                createPoll: { image, title, description, starts, ends }
            };

            const result = await client.execute(contract, handleMsg);
            pollId = result.logs[0].events[0].attributes.find(attr => attr.key === 'id').value;
        });

        describe('Successes', function () {
            it('should confirm poll creation success', async function () {
                const queryMsg = { getPoll: { id: pollId } };
                const result = await client.queryContractSmart(contract, queryMsg);
                expect(result.title).to.equal(title);
                expect(result.description).to.equal(description);
                expect(result.image).to.equal(image);
                expect(Number(result.startsAt)).to.equal(starts);
                expect(Number(result.endsAt)).to.equal(ends);
                expect(result.director).to.equal(deployer);
            });

            it('should confirm poll update success', async function () {
                const newTitle = 'Democratic Primary Election';
                const handleMsg = {
                    updatePoll: { id: pollId, image, title: newTitle, description, starts, ends }
                };
                await client.execute(contract, handleMsg);
                const queryMsg = { getPoll: { id: pollId } };
                const result = await client.queryContractSmart(contract, queryMsg);
                expect(result.title).to.equal(newTitle);
            });

            it('should confirm poll deletion success', async function () {
                const handleMsg = { deletePoll: { id: pollId } };
                await client.execute(contract, handleMsg);
                const queryMsg = { getPoll: { id: pollId } };
                const result = await client.queryContractSmart(contract, queryMsg);
                expect(result.deleted).to.be.true;
            });
        });

        describe('Failure', function () {
            it('should confirm poll creation failure', async function () {
                const handleMsg = { createPoll: { image: '', title, description, starts, ends } };
                await expectRevert(client.execute(contract, handleMsg), 'Image URL cannot be empty');
                handleMsg.createPoll = { image, title, description, starts: 0, ends };
                await expectRevert(client.execute(contract, handleMsg), 'Start date must be greater than 0');
            });

            it('should confirm poll update failure', async function () {
                const handleMsg = { updatePoll: { id: 100, image, title: 'New Title', description, starts, ends } };
                await expectRevert(client.execute(contract, handleMsg), 'Poll not found');
            });

            it('should confirm poll deletion failures', async function () {
                const handleMsg = { deletePoll: { id: 100 } };
                await expectRevert(client.execute(contract, handleMsg), 'Poll not found');
            });
        });
    });

    describe('Poll Contest', function () {
        beforeEach(async function () {
            const queryMsg = { getPolls: {} };
            const result = await client.queryContractSmart(contract, queryMsg);
            pollId = result[0].id;
        });

        describe('Success', function () {
            it('should confirm contest entry success', async function () {
                const handleMsg1 = { contest: { poll_id: pollId, name: name1, avatar: avatar1 } };
                await client.execute(contract, handleMsg1);
                const handleMsg2 = { contest: { poll_id: pollId, name: name2, avatar: avatar2 } };
                await client.execute(contract, handleMsg2);
                const queryMsg = { getContestants: { poll_id: pollId } };
                const contestants = await client.queryContractSmart(contract, queryMsg);
                expect(contestants).to.have.lengthOf(2);
            });
        });

        describe('Failure', function () {
            it('should confirm contest entry failure', async function () {
                const handleMsg = { contest: { poll_id: 100, name: name1, avatar: avatar1 } };
                await expectRevert(client.execute(contract, handleMsg), 'Poll not found');
                handleMsg.contest = { poll_id: pollId, name: '', avatar: avatar1 };
                await expectRevert(client.execute(contract, handleMsg), 'Name cannot be empty');
                const handleMsg1 = { contest: { poll_id: pollId, name: name1, avatar: avatar1 } };
                const handleMsg2 = { contest: { poll_id: pollId, name: name2, avatar: avatar2 } };
                await client.execute(contract, handleMsg1);
                await client.execute(contract, handleMsg2);
                await expectRevert(client.execute(contract, handleMsg1), 'Already contested');
            });
        });
    });

    describe('Poll Voting', function () {
        beforeEach(async function () {
            const queryMsg = { getPolls: {} };
            const result = await client.queryContractSmart(contract, queryMsg);
            pollId = result[0].id;
            const handleMsg1 = { contest: { poll_id: pollId, name: name1, avatar: avatar1 } };
            const handleMsg2 = { contest: { poll_id: pollId, name: name2, avatar: avatar2 } };
            await client.execute(contract, handleMsg1);
            await client.execute(contract, handleMsg2);
            const queryMsgContestants = { getContestants: { poll_id: pollId } };
            const contestants = await client.queryContractSmart(contract, queryMsgContestants);
            contestantId = contestants[0].id;
        });

        describe('Success', function () {
            it('should confirm voting success', async function () {
                const handleMsg1 = { vote: { poll_id: pollId, contestant_id: contestantId } };
                const handleMsg2 = { vote: { poll_id: pollId, contestant_id: contestantId } };
                await client.execute(contract, handleMsg1);
                await client.execute(contract, handleMsg2);
                const queryMsg = { getContestant: { poll_id: pollId, contestant_id: contestantId } };
                const result = await client.queryContractSmart(contract, queryMsg);
                expect(result.votes).to.be.equal(2);
            });
        });

        describe('Failure', function () {
            it('should confirm voting failure', async function () {
                const handleMsg = { vote: { poll_id: 100, contestant_id: contestantId } };
                await expectRevert(client.execute(contract, handleMsg), 'Poll not found');
                const handleMsgDeletePoll = { deletePoll: { id: pollId } };
                await client.execute(contract, handleMsgDeletePoll);
                await expectRevert(client.execute(contract, handleMsg), 'Polling not available');
            });
        });
    });
});



# Installtion And Setup

To set up the DappVotes project locally, follow these steps:

   # Clone the project
git clone https://github.com/Brig2002/d-voting-app.git
cd D-VOTING-APP

# Install dependencies
npm install  # or yarn install

# Configure environment (ensure configuration files are set up properly)

# Compile smart contracts (if applicable)
cargo build --release --target wasm32-unknown-unknown

# Upload smart contract to the network
secretcli tx compute store target/wasm32-unknown-unknown/release/your_contract.wasm --from your_wallet --gas 2000000 -y

# Get the code ID from the transaction response, then instantiate the contract
secretcli tx compute instantiate <code_id> '{"your": "init_msg"}' --from your_wallet --label "your_contract_label" --gas 200000 -y

# Run your application (if applicable)
npm start  # or yarn start
Contributing

If you would like to contribute to voting dApp, please follow these steps:


Fork the repository.
    Create a new branch (git checkout -b feature-branch).
    Make your changes.
    Commit your changes (git commit -am 'Add new feature').
    Push to the branch (git push origin feature-branch).
    Create a new Pull Request.

License

This project is licensed under the MIT License.
Contact

For any questions or feedback, please contact delalirock5@gmail.com, bamenorhu8@gmail.com.