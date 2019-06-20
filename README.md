# reportron
Generate pdf reports from json HTTP requests using .tex template files.


## Using reportron

Fire an HTTP POST request to `localhost:8000/generate` with body: 
```json
{
    "template": "test",
    "date": "20.06.2019"
}
```
