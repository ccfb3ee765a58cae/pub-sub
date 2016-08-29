var hosting = 'GitLab';
var repository = 'https://gitlab.com/creato/pub-sub/blob/master/';
var ourCrate = 'pub_sub';

$('.srclink').each(function(i, src) {
  var path = src.href.split('/src/').slice(1).join('/src/').split('/');
  if (path[0] === 'pub_sub') {
	var lines = path[path.length - 1].split('#')[1];
	var gitSrc = $('<a class="srclink-version-control"></a>')
	  .css('margin-right', '7px')
	  .attr('href', repository + path.slice(1).join('/').split('.html')[0] + '#L' + lines)
	  .text('[src on ' + hosting + ']');
	gitSrc.insertBefore(src);
  }
});
