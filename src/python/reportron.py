from Cheetah.Template import Template
import datetime
import os
from flask import Flask, request
app = Flask(__name__)


def readFile(filename):
    with open(filename) as filehandle:
        return filehandle.read()

def writeFile(filename, data):
    if not os.path.exists(os.path.dirname(filename)):
        try:
            os.makedirs(os.path.dirname(filename))
        except OSError as exc: # Guard against race condition
            if exc.errno != errno.EEXIST:
                raise

    with open(filename, "w") as filehandle:
        filehandle.write(data)

def generatePdf(templatename, data):
    # Get Tempalte definiton
    fileDir = os.path.dirname(os.path.realpath('__file__'))
    print('Script dir: {}'.format(fileDir))
    tempalteFile = os.path.join(fileDir, '../../templates/fantapptic_invoice.tex')
    tempalteFile = os.path.abspath(os.path.realpath(tempalteFile))
    
    templateDef = readFile(tempalteFile)
      
    # Output .tex
    outputtex = Template(templateDef, searchList=[data])
    tempalteFileOutput = os.path.join(fileDir, '../../pdf/temp/fantapptic_invoice.tex')
    tempalteFileOutput = os.path.abspath(os.path.realpath(tempalteFileOutput))
    writeFile(tempalteFileOutput, str(outputtex))
    
    # Convert .tex to .pdf
    pdfOutput = os.path.join(fileDir, '../../pdf')
    pdfOutput = os.path.abspath(os.path.realpath(pdfOutput))
    command = "pdflatex -output-directory={} {}".format(pdfOutput, tempalteFileOutput)
    print("Command: {}".format(command))
    os.system(command)
    pdfOutput = os.path.join(pdfOutput, "fantapptic_invoice.pdf")
    print("Finished: {}".format(pdfOutput))
    return str(pdfOutput)


@app.route("/")
def hello():
    return "Hello World!"

@app.route('/generate', methods=['POST'])
def generate():
    content = request.get_json(silent=True)
    return content

