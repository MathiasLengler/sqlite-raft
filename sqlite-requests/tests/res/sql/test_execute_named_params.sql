DELETE FROM countries WHERE countries.alpha_2 LIKE (:alpha_2) AND countries.rank < (:rank)