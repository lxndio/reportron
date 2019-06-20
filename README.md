# reportron
Generate pdf reports from json HTTP requests using .tex template files.


## using reportron

Fire a HTTP POST request at `localhost:8000/generate` with body: 
```json
{
    "template": "test",
    "date": "20.06.2019"
}
```
