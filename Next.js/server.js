const express = require('express');
const cors = require('cors');
const app = express();
const port = 3001;

app.use(cors({
  origin: 'http://localhost:3000' // Replace with your frontend URL
}));

app.get('/api/vote', (req, res) => {
  res.send({ message: 'Vote recorded' });
});

app.listen(port, () => {
  console.log(`Backend listening at http://localhost:${port}`);
});
