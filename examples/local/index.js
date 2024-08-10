document.addEventListener('DOMContentLoaded', function() {
  let elm = document.getElementById('js')
  let original = elm.innerHTML

  // Randomly change the text every 500 ms

  setInterval(function() {
    let text = original.split(' ')
    let newText = text.map(function(word) {
      if (Math.random() > 0.5) {
        return word.toUpperCase()
      } else {
        return word.toLowerCase()
      }
    }).join(' ')

    elm.innerHTML = newText
  }, 500)
})