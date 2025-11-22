/**
 * GNU Terry Pratchett Plugin
 *
 * Adds the X-Clacks-Overhead: GNU Terry Pratchett header to all pages
 * as a tribute to Sir Terry Pratchett.
 *
 * Learn more: http://www.gnuterrypratchett.com/
 */

module.exports = function (context, options) {
  return {
    name: 'clacks-overhead-plugin',

    injectHtmlTags() {
      return {
        headTags: [
          {
            tagName: 'meta',
            attributes: {
              'http-equiv': 'X-Clacks-Overhead',
              content: 'GNU Terry Pratchett',
            },
          },
        ],
      };
    },
  };
};
