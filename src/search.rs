// TODO: Implement the search functionality into this file
// For each term in the search query loop trough every document that has the term
// 1. Calculate term t frequency f in document d
// TF(t, d) = count(t) / d.terms.length
// (count(t) is precalculated in the database in frequency col)
// term, comicNum, frequency
// 2. Calculate document d frequency F
// IDF(tm D) = log(count(documents) / count(documents with term t))
// count(documents) can be stored as constant on every index
// then you can do SELECT COUNT(term) FROM terms WHERE term = 't'
// 3. rank each document by TF(t, d) * IDF(t, D) and return the top N results
