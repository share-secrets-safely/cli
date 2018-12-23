{{ page.title }}

{% if user %}
  Hello {{ user.name }}!
{% endif %}

{{ "/my/fancy/url" | append: ".html" }}


{% if product.title contains 'Pack' %}
  This product's title contains the word Pack.
{% endif %}



{% assign my_string = "Hello World!" %}


{% assign my_int = 25 %}
{% assign my_float = 39.756 %}


{% assign foo = true %}
{% assign bar = false %}

{% for user in site.users %}
  {{ user }}
{% endfor %}


{% assign my_variable = "tomato" %}
{{ my_variable }}

{%- assign my_variable = "tomato" -%}
{{ my_variable }}

Anything you put between {% comment %} and {% endcomment %} tags
is turned into a comment.



{% unless product.title == 'Awesome Shoes' %}
  These shoes are not awesome.
{% endunless %}

{% if product.title != 'Awesome Shoes' %}
  These shoes are not awesome.
{% endif %}

{% if customer.name == 'kevin' %}
  Hey Kevin!
{% elsif customer.name == 'anonymous' %}
  Hey Anonymous!
{% else %}
  Hi Stranger!
{% endif %}

{% assign handle = 'cake' %}
{% case handle %}
  {% when 'cake' %}
     This is a cake
  {% when 'cookie' %}
     This is a cookie
  {% else %}
     This is not a cake nor a cookie
{% endcase %}


{% for i in (1..5) %}
  {% if i == 4 %}
    {% break %}
  {% else %}
    {{ i }}
  {% endif %}
{% endfor %}


{% for i in (1..5) %}
  {% if i == 4 %}
    {% continue %}
  {% else %}
    {{ i }}
  {% endif %}
{% endfor %}

{% for item in array limit:2 %}
  {{ item }}
{%- endfor -%}

{% for i in (3..5) %}
  {{ i }}
{% endfor %}

{% assign num = 4 %}
{% for i in (1..num) %}
  {{ i }}
{% endfor %}

{% for item in array reversed %}
  {{ item }}
{% endfor %}


{% raw %}
  In Handlebars, {{ this }} will be HTML-escaped, but
  {{ that }}} will not.
{% endraw %}

{% capture my_variable %}I am being captured.{% endcapture %}
{{ my_variable }}


{% assign favorite_food = 'pizza' %}
{% assign age = 35 %}

{% capture about_me %}
I am {{ age }} and my favorite food is {{ favorite_food }}.
{% endcapture %}

{{ about_me }}



{{ -17 | abs }}
{{ "/my/fancy/url" | append: ".html" }}
{{ "title" | capitalize }}
{{ 1.2 | ceil }}
{{ "3.5" | ceil }}
{% assign site_categories = site.pages | map: 'category' | compact %}

{% for category in site_categories %}
  {{ category }}
{% endfor %}

{% assign fruits = "apples, oranges, peaches" | split: ", " %}
{% assign vegetables = "carrots, turnips, potatoes" | split: ", " %}

{% assign everything = fruits | concat: vegetables %}

{% for item in everything %}
- {{ item }}
{% endfor %}

{% assign furniture = "chairs, tables, shelves" | split: ", " %}

{% assign everything = fruits | concat: vegetables | concat: furniture %}

{% for item in everything %}
- {{ item }}
{% endfor %}

{{ "March 25, 2018" | date: "%b %d, %y" }}
{{ null | default: 2.99 }}

{% assign product_price = "" %}
{{ product_price | default: 2.99 }}
{{ 16 | divided_by: 4 }}

{% assign my_integer = 7 %}
{{ 20 | divided_by: my_integer }}

{% assign my_integer = 7 %}
{% assign my_float = my_integer | times: 1.0 %}
{{ 20 | divided_by: my_float }}
{{ "Parker Moore" | downcase }}
{{ "Have you read 'James & the Giant Peach'?" | escape }}
{{ "1 &lt; 2 &amp; 3" | escape_once }}

{% assign my_array = "zebra, octopus, giraffe, tiger" | split: ", " %}

{{ 1.2 | floor }}
{{ "3.5" | floor }}
{% assign beatles = "John, Paul, George, Ringo" | split: ", " %}
{{ beatles | join: " and " }}
{{ "          So much room for activities!          " | lstrip }}
{{ 4 | minus: 2 }}
{{ 3 | modulo: 2 }}
{% capture string_with_newlines %}
Hello
there
{% endcapture %}
{{ string_with_newlines | newline_to_br }}
{{ 4 | plus: 2 }}
{{ "apples, oranges, and bananas" | prepend: "Some fruit: " }}
{{ "I strained to see the train through the rain" | remove: "rain" }}
{{ "I strained to see the train through the rain" | remove_first: "rain" }}
{{ "Take my protein pills and put my helmet on" | replace: "my", "your" }}
{% assign my_string = "Take my protein pills and put my helmet on" %}

{{ my_string | replace_first: "my", "your" }}
{{ "Ground control to Major Tom." | split: "" | reverse | join: "" }}

{% assign my_array = "apples, oranges, peaches, plums" | split: ", " %}
{{ my_array | reverse | join: ", " }}

{{ 1.2 | round }}

{{ "          So much room for activities!          " | rstrip }}

{{ "Ground control to Major Tom." | size }}

{% assign my_array = "apples, oranges, peaches, plums" | split: ", " %}
{{ my_array | size }}

{{ "Liquid" | slice: 2, 5 }}
{{ "Liquid" | slice: -3, 2 }}

{% assign my_array = "zebra, octopus, giraffe, Sally Snake" | split: ", " %}
{{ my_array | sort | join: ", " }}

{% assign my_array = "zebra, octopus, giraffe, Sally Snake" | split: ", " %}
{{ my_array | sort_natural | join: ", " }}

{{ "          So much room for activities!          " | strip }}

{{ "Have <em>you</em> read <strong>Ulysses</strong>?" | strip_html }}

{% capture string_with_newlines %}
Hello
there
{% endcapture %}

{{ string_with_newlines | strip_newlines }}
{{ 3 | times: 2 }}
{{ "Ground control to Major Tom." | truncate: 25, ", and so on" }}
{{ "Ground control to Major Tom." | truncatewords: 3 }}

{% assign my_array = "ants, bugs, bees, bugs, ants" | split: ", " %}
{{ my_array | uniq | join: ", " }}
{{ "some text as base64" | base64 }}
