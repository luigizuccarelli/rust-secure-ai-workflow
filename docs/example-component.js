{
  "type": "exported_components",
  "flowstreamid": "fJ8bcyW1cE61f",
  "items": [
    {
      "id": "imexv86ud",
      "component": "c3da1288ff",
      "name": "exec",
      "config": {},
      "inputs": [
        {
          "id": "input",
          "name": "Input"
        }
      ],
      "outputs": [
        {
          "id": "output",
          "name": "Output"
        },
        {
          "id": "error",
          "name": "Error"
        }
      ],
      "connections": {
        "output": [
          {
            "id": "imexva86d",
            "index": "input"
          }
        ],
        "error": [
          {
            "id": "imexvalla",
            "index": "input"
          }
        ]
      },
      "x": 342,
      "y": 534,
      "$inputs": true,
      "$outputs": false
    }
  ],
  "components": {
    "c3da1288ff": {
      "name": "Exec",
      "source": "<script total>\n\n\texports.id = 'c3da1288ff';\n\texports.name = 'Exec';\n\texports.icon = 'ti ti-code';\n\texports.author = 'Total.js';\n\texports.version = '1';\n\texports.group = 'Common';\n\texports.config = {};\n\texports.inputs = [{ id: 'input', name: 'Input' }];\n\texports.outputs = [{ id: 'output', name: 'Output' }, { id: 'error', name: 'Error' }];\n\n\tconst { exec } = require(\"child_process\");\n\n\texports.make = function(instance, config) {\n\n\t\t// instance.main.variables {Object}\n\t\t// instance.main.variables2 {Object}\n\t\t// instance.save();\n\t\t// instance.replace(str); // replaces {variable_name} for values from \"variables\" and \"variables2\"\n\t\t// instance.status(obj, [refresh_delay_in_ms]);\n\n\t\tinstance.message = function($) {\n\n\t\t\tvar data = $.data\n\t\t\texec(data, (error, stdout, stderr) => {\n\t\t\t\tif (error) {\n\t\t\t\t\t//console.log(`error: ${error.message}`);\n\t\t\t\t\t$.send('error',error.message);\n\t\t\t\t}\n\t\t\t\tif (stderr) {\n\t\t\t\t\t//console.log(`stderr: ${stderr}`);\n\t\t\t\t\t$.send('error',stderr);\n\t\t\t\t}\n\n\t\t\t\t$.send('output',stdout)\n\t\t\t\t//console.log(`stdout: ${stdout}`);\n\t\t\t});\n\t\t};\n\n\t\tinstance.configure = function() {\n\t\t\t// \"config\" is changed\n\t\t};\n\n\t\tinstance.close = function() {\n\t\t\t// this instance is closed\n\t\t};\n\n\t\tinstance.vary = function(type) {\n\t\t\t// @type {String} variables|variables2|secrets\n\t\t\t// FlowStream variables are changed\n\t\t};\n\n\t\tinstance.configure();\n\n\t};\n\n</script>\n\n<readme>\nMarkdown readme\n\n```js\nvar total = 'Hello world!';\n```\n</readme>\n\n<settings>\n\t<div class=\"padding\">\n\t\tSETTINGS for this component (optional)\n\t</div>\n</settings>\n\n<style>\n\t.CLASS footer { padding: 10px; font-size: 12px; }\n</style>\n\n<script>\n\n\t// Client-side script\n\t// Optional, you can remove it\n\n\t// A custom helper for the component instances\n\t// The method below captures each instance of this component\n\tTOUCH(function(exports, reinit) {\n\n\t\tvar name = exports.name + ' --> ' + exports.id;\n\n\t\tconsole.log(name, 'initialized' + (reinit ? ' : UPDATE' : ''));\n\n\t\texports.settings = function(meta) {\n\t\t\t// Triggered when the user opens settings\n\t\t\tconsole.log(name, 'settings', meta);\n\t\t};\n\n\t\texports.configure = function(config, isinit) {\n\t\t\t// Triggered when the config is changed\n\t\t\tconsole.log(name, 'configure', config);\n\t\t};\n\n\t\texports.status = function(status, isinit) {\n\t\t\t// Triggered when the status is changed\n\t\t\tconsole.log(name, 'status', status);\n\t\t};\n\n\t\texports.note = function(note, isinit) {\n\t\t\t// Triggered when the note is changed\n\t\t\tconsole.log(name, 'note', note);\n\t\t};\n\n\t\texports.variables = function(variables) {\n\t\t\t// Triggered when the variables are changed\n\t\t\tconsole.log(name, 'variables', variables);\n\t\t};\n\n\t\texports.variables2 = function(variables) {\n\t\t\t// Triggered when the variables2 are changed\n\t\t\tconsole.log(name, 'variables2', variables);\n\t\t};\n\n\t\texports.redraw = function() {\n\t\t\t// Flow design has been redrawn\n\t\t\tconsole.log(name, 'redraw');\n\t\t};\n\n\t\texports.move = function() {\n\t\t\t// Instance has changed position\n\t\t\tconsole.log(name, 'move');\n\t\t};\n\n\t\texports.close = function() {\n\t\t\t// Triggered when the instance is closing due to some reasons\n\t\t\tconsole.log(name, 'close');\n\t\t};\n\n\t});\n\n</script>\n\n<body>\n\t<header>\n\t\t<i class=\"$ICON\"></i>$NAME\n\t</header>\n\t<footer>lmz</footer>\n</body>"
    }
  }
}
